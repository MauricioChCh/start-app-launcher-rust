// ============================================================================
// IMPORTS 
// ============================================================================
// Nota primera vez en usar rsut por lo que hay muchos comentarios explicativos
// En Rust, los imports se hacen con "use" en lugar de "import" o "include"

use ratatui::{
    backend::CrosstermBackend,           // Backend para manejar la terminal
    crossterm::{                          // Librería para control de terminal
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,                          // Macro para ejecutar comandos en terminal
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Alignment, Constraint, Direction, Layout},  // Para organizar elementos en pantalla
    style::{Color, Modifier, Style},     // Estilos: colores, negritas, etc
    text::{Line, Span},                   // Texto estructurado
    widgets::{Block, Borders, List, ListItem, Paragraph}, // Widgets de UI
    Terminal,                             // Terminal principal
};
use std::io;                              // Entrada/salida estándar
use std::process::Command;                // Para ejecutar comandos del sistema

// ============================================================================
// STRUCT App 
// ============================================================================
// En Rust, los "structs" son como registros o estructuras de datos
// No tienen métodos dentro, los métodos van en un bloque "impl" aparte

struct App {
    options: Vec<&'static str>,  // Vec = vector dinámico (como vector<> en C++)
                                 // &'static str = string literal que vive todo el programa
    selected: usize,             // usize = unsigned integer (como size_t en C++, osea usado para conteos de objetos en memoria)
}

// ============================================================================
// BLOQUE "impl" - Aquí van los métodos de App 
// ============================================================================
impl App {
    /// Crea una nueva instancia de App (constructor)
    /// En Rust la convención es usar "new()"
    fn new() -> Self {  // Self = el tipo App mismo, es como "App" explícitamente
        App {
            options: vec!["Nothing", "Study", "Docker", "Dev", "Play"],
            selected: 0,
        }
    }

    /// Selecciona la siguiente opción (circular)
    /// &mut self = referencia mutable (como "this" en C++, pero explícitamente mutable)
    /// En Rust, por defecto todo es inmutable. Necesitas &mut para modificar
    fn next(&mut self) {
        // (a + 1) % len da efecto circular: después del último vuelve al primero
        self.selected = (self.selected + 1) % self.options.len();
    }

    /// Selecciona la opción anterior (circular)
    fn prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            // Si estamos en la primera, vamos a la última
            self.selected = self.options.len() - 1;
        }
    }

    /// Ejecuta el comando según la opción seleccionada
    /// &self = referencia inmutable (no modificamos la estructura)
    // TODO: Mejorar manejo de errores
    // TODO : Hacer que las opciones y comandos sean configurables 
    fn select(&self) {
        // match = switch en C++/JavaScript, pero más poderoso
        // Aquí comparamos self.options[self.selected] con cada caso
        match self.options[self.selected] {
            "Nothing" => {},  // {} vacío = no hacer nada
            
            "Study" => {
                // Command::new() crea un proceso nuevo (como fork + exec en C)
                // spawn() lo inicia en background
                // .ok() ignora errores (parecido a try-catch pero más limpio)
                Command::new("obsidian").spawn().ok();
                Command::new("brave-browser").spawn().ok();
            }
            
            "Docker" => {
                // arg("-c") y arg("comando") construyen un comando shell
                // Equivalente a: sh -c "docker start $(docker ps -aq)"
                Command::new("sh")
                    .arg("-c")
                    .arg("docker start $(docker ps -aq)")
                    .spawn()
                    .ok();
                Command::new("konsole").spawn().ok();
                Command::new("obsidian").spawn().ok();
            }
            
            "Dev" => {
                Command::new("code").spawn().ok();
                Command::new("obsidian").spawn().ok();
            }
            
            "Play" => {
                Command::new("discord").spawn().ok();
                Command::new("steam").spawn().ok();
            }
            
            _ => {}  // _ = "cualquier otro caso" (como default: en switch)
        }
    }
}

// ============================================================================
// FUNCIÓN main - Punto de entrada (como main() en C++)
// ============================================================================
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Result = tipo que puede ser Ok(valor) o Err(error)
    // Box<dyn std::error::Error> = cualquier tipo de error (similar a Exception en Java)
    // El ? al final de operaciones propaga errores automáticamente (como throw en C++)

    // ========== SETUP DE LA TERMINAL ==========
    
    // enable_raw_mode() = que la terminal capture cada tecla inmediatamente
    // Sin esto, necesitarías presionar Enter para enviar entrada
    enable_raw_mode()?;
    
    let mut stdout = io::stdout();  // Obtener salida estándar
                                    // mut = mutable (es la única forma de modificarlo)
    
    // execute! = macro que ejecuta comandos en la terminal
    // EnterAlternateScreen = cambiar a pantalla alternativa (como vim/less hacen)
    // EnableMouseCapture = capturar clics del mouse
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    // CrosstermBackend = configurar que Ratatui use Crossterm para la terminal
    let backend = CrosstermBackend::new(stdout);
    
    // Terminal::new() = crear terminal
    // &mut terminal = referencia mutable para poder dibujar en ella
    let mut terminal = Terminal::new(backend)?;

    // Crear la aplicación
    let app = App::new();
    
    // Ejecutar el loop principal
    // Si hay error, guardarlo en res
    let res = run_app(&mut terminal, app);

    // ========== CLEANUP - Restaurar terminal a estado anterior ==========
    
    disable_raw_mode()?;
    
    execute!(
        terminal.backend_mut(),  // backend_mut() = acceder al backend mutablemente
        LeaveAlternateScreen,    // Volver a pantalla normal
        DisableMouseCapture      // Dejar de capturar mouse
    )?;
    
    terminal.show_cursor()?;     // Mostrar el cursor (estaba oculto)

    // Si hubo error en run_app(), imprimirlo
    if let Err(err) = res {
        // if let = destructuring (similar a pattern matching en Kotlin)
        println!("{:?}", err);   // {:?} = formato de debug
    }

    Ok(())  // Retornar Ok (sin error)
}

// ============================================================================
// FUNCIÓN run_app - Loop principal
// ============================================================================
// Esta función contiene el "game loop" de la aplicación
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,  // Terminal mutable
    mut app: App  // App mutable (necesita ser mutable porque cambias selected)
) -> io::Result<()> {
    // io::Result<()> = puede ser Ok(()) o Err(io::Error)
    
    loop {  // Loop infinito (como while(true) en C++)
        
        // ========== DIBUJAR LA UI ==========
        
        // terminal.draw(|f| ...) = closure (lambda en JavaScript, función anónima en C++)
        // f = referencia al Frame donde dibujamos
        terminal.draw(|f| ui(f, &app))?;
        
        // ========== LEER EVENTOS (teclado/mouse) ==========
        
        // event::poll() = esperar hasta timeout (250ms) a que ocurra un evento
        // ? = si hay error, salir de la función con ese error
        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            
            // event::read() = leer el evento que ocurrió
            if let Event::Key(key) = event::read()? {
                // if let = solo procesar si es un evento de teclado
                
                // match = comparar el código de tecla
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // | = OR, así que 'q' O Esc hacen lo mismo
                        return Ok(());  // Salir del loop (terminar el programa)
                    }
                    
                    KeyCode::Down | KeyCode::Char('j') => {
                        // Flecha abajo o 'j' (vim-style navigation)
                        app.next();
                    }
                    
                    KeyCode::Up | KeyCode::Char('k') => {
                        // Flecha arriba o 'k' (vim-style navigation)
                        app.prev();
                    }
                    
                    KeyCode::Enter => {
                        // Enter = ejecutar la opción seleccionada
                        app.select();
                        return Ok(());  // Salir después de ejecutar
                    }
                    
                    _ => {}  // Ignorar todas las otras teclas
                }
            }
        }
    }
}

// ============================================================================
// FUNCIÓN ui - Renderizar la interfaz
// ============================================================================
// &mut ratatui::Frame = referencia mutable al Frame (donde dibujamos)
// &App = referencia inmutable a la app (solo lectura)
fn ui(f: &mut ratatui::Frame, app: &App) {
    // f.area() = obtener tamaño de la pantalla (rectángulo)
    let size = f.area();

    // ========== LAYOUT ==========
    
    // Dividir la pantalla en 3 secciones verticales
    // Layout::default() = usar configuración por defecto
    let chunks = Layout::default()
        .direction(Direction::Vertical)  // Dividir verticalmente
        .margin(2)                        // Margen de 2 espacios
        .constraints(
            [
                Constraint::Length(3),    // Sección 1: 3 líneas (título)
                Constraint::Min(10),      // Sección 2: mínimo 10 líneas (lista)
                Constraint::Length(3),    // Sección 3: 3 líneas (controles)
            ]
            .as_ref(),  // Convertir array a slice
        )
        .split(size);  // Dividir el área según las constraints

    // ========== SECCIÓN 1: TÍTULO ==========
    
    let title = Paragraph::new("What are you going to do today?")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    
    f.render_widget(title, chunks[0]);  // Dibujar en la sección 1

    // ========== SECCIÓN 2: LISTA DE OPCIONES ==========
    
    // Convertir vector de opciones a vector de ListItem (widgets renderizables)
    let items: Vec<ListItem> = app
        .options
        .iter()          // Iterar sobre las opciones
        .enumerate()     // Obtener (índice, valor)
        .map(|(i, option)| {  // Para cada opción, crear un ListItem
            
            // Determinar estilo: si está seleccionada, destacar
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Black)           // Texto negro
                    .bg(Color::Cyan)            // Fondo cyan
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)  // Texto blanco normal
            };

            // Crear la línea de texto: "  ▸ Option"
            let line = Line::from(Span::styled(
                format!("  ▸ {}", option),  // format! = sprintf en C++
                style,
            ));

            ListItem::new(line)  // Convertir en ListItem
        })
        .collect();  // Recopilar todos en un Vec

    // Crear el widget List con los items
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)    // Dibujar borde
                .title(" Options ")       // Título del borde
                .border_type(ratatui::widgets::BorderType::Rounded)  // Bordes redondeados
                .style(Style::default().fg(Color::Cyan))
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(list, chunks[1]);  // Dibujar en la sección 2

    // ========== SECCIÓN 3: FOOTER CON CONTROLES ==========
    
    let footer = Paragraph::new("↑/k: Up  |  ↓/j: Down  |  Enter: Select  |  q/Esc: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    
    f.render_widget(footer, chunks[2]);  // Dibujar en la sección 3
}