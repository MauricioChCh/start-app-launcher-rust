// ============================================================================
// mkdir -p ~/.config/launcher/
// ============================================================================
// ============================================================================
// IMPORTS
// ============================================================================
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;
use std::process::Command;

// ============================================================================
// STRUCTS - Configuración y App
// ============================================================================

/// Representa una aplicación dentro de un grupo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCommand {
    /// Nombre mostrado en la UI
    pub name: String,
    /// Comando a ejecutar
    pub command: String,
    /// Argumentos opcionales (ej: ["-c", "docker start $(docker ps -aq)"])
    #[serde(default)]
    pub args: Vec<String>,
    /// Si true, usa `sh -c` para ejecutar (para comandos complejos)
    #[serde(default)]
    pub use_shell: bool,
}

/// Un grupo de aplicaciones a ejecutar juntas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub apps: Vec<AppCommand>,
}

/// Configuración general del launcher
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub groups: Vec<Group>,
}

impl Config {
    /// Cargar configuración desde un archivo JSON
    pub fn load(path: &PathBuf) -> io::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Cargar configuración desde ubicación estándar
    /// 1. `./launcher.json`
    /// 2. `~/.config/launcher/config.json`
    /// 3. `/etc/launcher/config.json`
    pub fn load_default() -> io::Result<Self> {
        let paths = [
            PathBuf::from("./launcher.json"),
            dirs::config_dir()
                .map(|d| d.join("launcher").join("config.json"))
                .unwrap_or_default(),
            PathBuf::from("/etc/launcher/config.json"),
        ];

        for path in &paths {
            if path.exists() {
                return Self::load(path);
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No config file found. Create launcher.json in current directory.",
        ))
    }
}

/// Estructura principal de la aplicación
struct App {
    groups: Vec<String>,  // Nombres de grupos
    selected: usize,
    config: Config,
}

impl App {
    /// Crear nueva instancia desde configuración
    fn new(config: Config) -> Self {
        // Extraer solo los nombres de los grupos para la UI
        let groups = config.groups.iter().map(|g| g.name.clone()).collect();
        App {
            groups,
            selected: 0,
            config,
        }
    }

    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.groups.len();
    }

    fn prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.groups.len().saturating_sub(1);
        }
    }

    /// Ejecutar todas las aplicaciones del grupo seleccionado
    fn select(&self) {
        // Verificar que selected es válido
        if self.selected >= self.config.groups.len() {
            return;
        }

        // Acceder al grupo seleccionado desde la configuración        
        let group = &self.config.groups[self.selected];
        // Ejecutar cada aplicación del grupo (Esto evita problemas de borrow)
        for app in &group.apps {
            Self::execute_command(app);
        }
    }

    /// Ejecutar un comando individual de forma desacoplada de la terminal
    fn execute_command(app: &AppCommand) {
        let child = if app.use_shell {
            // Para comandos complejos con pipes, variables, etc
            Command::new("sh")
                .arg("-c")
                .args(&app.args)
                .spawn()
        } else {
            // Para comandos simples
            let mut cmd = Command::new(&app.command);
            cmd.args(&app.args);
            
            // Importante: desacoplar del padre para que la app no muera
            // cuando cierre la terminal
            // Esto crea una nueva sesión de proceso con setsid()
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                unsafe {
                    cmd.pre_exec(|| {
                        // Cambiar a nuevo session group
                        libc::setsid();
                        Ok(())
                    });
                }
            }
            cmd.spawn()
        };

        if let Err(e) = child {
            eprintln!("Error al ejecutar {}: {}", app.name, e);
        }
    }
}

// ============================================================================
// FUNCIÓN main
// ============================================================================
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Cargar configuración
    let config = Config::load_default()?;

    if config.groups.is_empty() {
        eprintln!("Error: No groups configured in config file");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(config);
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

// ============================================================================
// FUNCIÓN run_app
// ============================================================================
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(());
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.prev();
                    }
                    KeyCode::Enter => {
                        app.select();
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}

// ============================================================================
// FUNCIÓN ui
// ============================================================================
fn ui(f: &mut ratatui::Frame, app: &App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(size);

    // Título
    let title = Paragraph::new("What are you going to do today?")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Lista de grupos
    let items: Vec<ListItem> = app
        .groups
        .iter()
        .enumerate()
        .map(|(i, group)| {
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let line = Line::from(Span::styled(format!("  ▸ {}", group), style));
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Group Apps ")
                .border_type(ratatui::widgets::BorderType::Rounded)
                .style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(list, chunks[1]);

    // Footer
    let footer = Paragraph::new("↑/k: Up  |  ↓/j: Down  |  Enter: Select  |  q/Esc: Quit")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}