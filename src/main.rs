use clap::Parser;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::PathBuf;

/// Application to calculate and display the SHA-256 hash of a file.
///
/// This application can run in terminal mode or with a graphical interface,
/// depending on how it is invoked.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the file for which the hash will be calculated
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Force command line mode
    #[arg(short, long)]
    cli: bool,
}

/// Calculates the SHA-256 hash of a file.
///
/// # Arguments
///
/// * `path` - Path to the file for which the hash will be calculated
///
/// # Returns
///
/// * `io::Result<String>` - The hash in hexadecimal format or an error
fn calculate_hash(path: &PathBuf) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

/// Main function of the application in CLI mode.
///
/// Processes command line arguments and displays the hash.
fn run_cli(file_path: PathBuf) -> io::Result<()> {
    println!("Calculating hash for: {}", file_path.display());
    
    match calculate_hash(&file_path) {
        Ok(hash) => {
            println!("SHA-256 Hash: {}", hash);
            Ok(())
        },
        Err(e) => {
            eprintln!("Error calculating hash: {}", e);
            Err(e)
        }
    }
}

#[cfg(feature = "gui")]
mod gui {
    use super::*;
    use eframe::{egui, App, CreationContext, Theme};
    use rfd::FileDialog;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::thread;

    pub struct HashApp {
        selected_file: Option<PathBuf>,
        hash_result: Option<Result<String, String>>,
        calculating: bool,
        rx: Option<Receiver<Result<String, String>>>,
        tx: Option<Sender<()>>,
        animation_time: f32,
    }

    impl Default for HashApp {
        fn default() -> Self {
            Self {
                selected_file: None,
                hash_result: None,
                calculating: false,
                rx: None,
                tx: None,
                animation_time: 0.0,
            }
        }
    }

    impl App for HashApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            // Use dark theme by default, but follow system configuration
            ctx.set_visuals(if ctx.style().visuals.dark_mode {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            
            // Increment animation time for other elements, but not for the title
            self.animation_time += ctx.input(|i| i.unstable_dt).min(0.1) as f32;

            egui::CentralPanel::default().show(ctx, |ui| {
                // Large title with fixed style (no animation)
                ui.vertical_centered(|ui| {
                    // Main title with fixed size
                    ui.add_space(20.0);
                    ui.heading(
                        egui::RichText::new("HashSafe")
                            .size(32.0)
                            .strong()
                            .color(if ui.visuals().dark_mode {
                                egui::Color32::from_rgb(220, 220, 220)
                            } else {
                                egui::Color32::from_rgb(50, 50, 50)
                            })
                    );
                    
                    // Subtitle with theme-adaptable color
                    ui.label(
                        egui::RichText::new("File Hash Calculator")
                            .size(16.0)
                            .color(if ui.visuals().dark_mode {
                                egui::Color32::from_rgb(180, 180, 180)
                            } else {
                                egui::Color32::from_rgb(100, 100, 100)
                            })
                    );
                });
                
                // Add theme selector
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    let mut dark_mode = ui.visuals().dark_mode;
                    if ui.radio_value(&mut dark_mode, true, "Dark").clicked() {
                        ctx.set_visuals(egui::Visuals::dark());
                    }
                    if ui.radio_value(&mut dark_mode, false, "Light").clicked() {
                        ctx.set_visuals(egui::Visuals::light());
                    }
                });
                
                ui.add_space(20.0);
                
                // macOS style button to select file
                ui.vertical_centered(|ui| {
                    let button_response = ui.add(egui::Button::new(
                        egui::RichText::new("Select File")
                            .size(18.0)
                    ).min_size(egui::vec2(180.0, 40.0)));
                    
                    // macOS style hover and click effect
                    if button_response.clicked() {
                        if let Some(path) = FileDialog::new().pick_file() {
                            self.selected_file = Some(path);
                            self.hash_result = None;
                        }
                    }
                });

                // Show only the filename (not the full path) with proper handling of special characters
                if let Some(path) = &self.selected_file {
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        let fade_in = (self.animation_time * 2.0).min(1.0);
                        
                        // Use OsString directly and convert it to a valid UTF-8 representation
                        let file_name = path.file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "Unknown file".to_string());
                        
                        // Get file extension to determine the type
                        let extension = path.extension()
                            .map(|ext| ext.to_string_lossy().into_owned().to_lowercase())
                            .unwrap_or_else(|| "".to_string());
                            
                        // Show filename with extension icon
                        ui.horizontal(|ui| {
                            // Try to show a basic file type indicator based on extension
                            let (icon, color) = match extension.as_str() {
                                "txt" | "md" | "rtf" => ("📄", egui::Color32::from_rgb(120, 120, 220)),
                                "pdf" => ("📑", egui::Color32::from_rgb(220, 80, 80)),
                                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" => ("🖼️", egui::Color32::from_rgb(80, 180, 80)),
                                "mp3" | "wav" | "ogg" | "flac" => ("🎵", egui::Color32::from_rgb(180, 120, 180)),
                                "mp4" | "avi" | "mov" | "mkv" => ("🎬", egui::Color32::from_rgb(120, 180, 220)),
                                "zip" | "tar" | "gz" | "7z" | "rar" => ("🗜️", egui::Color32::from_rgb(180, 160, 80)),
                                "exe" | "app" | "dmg" => ("📦", egui::Color32::from_rgb(200, 100, 100)),
                                "html" | "css" | "js" => ("🌐", egui::Color32::from_rgb(100, 180, 200)),
                                "py" | "rs" | "c" | "cpp" | "java" => ("📝", egui::Color32::from_rgb(120, 200, 120)),
                                _ => ("📄", egui::Color32::from_rgb(150, 150, 150)),
                            };
                            
                            ui.label(
                                egui::RichText::new(icon)
                                    .size(16.0)
                                    .color(color)
                            );
                            
                            // Use strong() to ensure special characters are displayed correctly
                            let text = egui::RichText::new(&file_name)
                                .size(14.0)
                                .strong()
                                .color(if ui.visuals().dark_mode {
                                    egui::Color32::from_rgba_premultiplied(
                                        220, 220, 220, (fade_in * 255.0) as u8
                                    )
                                } else {
                                    egui::Color32::from_rgba_premultiplied(
                                        70, 70, 70, (fade_in * 255.0) as u8
                                    )
                                });
                                
                            // Use text widget instead of label for better rendering control
                            ui.add(egui::Label::new(text).wrap(false));
                        });
                    });
                }

                ui.add_space(20.0);

                // Rest of the interface to calculate hash
                if let Some(path) = &self.selected_file {
                    // macOS style button to calculate hash
                    ui.vertical_centered(|ui| {
                        if !self.calculating && ui.add(egui::Button::new(
                            egui::RichText::new("Calculate Hash")
                                .size(16.0)
                        ).min_size(egui::vec2(150.0, 36.0))).clicked() {
                            let path_clone = path.clone();
                            self.calculating = true;
                            
                            let (result_tx, result_rx) = channel();
                            let (cancel_tx, cancel_rx) = channel();
                            
                            self.rx = Some(result_rx);
                            self.tx = Some(cancel_tx);
                            
                            thread::spawn(move || {
                                let result = calculate_hash(&path_clone);
                                // Check if calculation was cancelled
                                if cancel_rx.try_recv().is_ok() {
                                    return;
                                }
                                let _ = result_tx.send(result.map_err(|e| e.to_string()));
                            });
                        }
                    });
                }

                // Show loading animation during calculation
                if self.calculating {
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        // macOS style spinner
                        let spinner_angle = self.animation_time * 5.0;
                        let spinner_radius = 10.0;
                        let center = ui.next_widget_position() + egui::vec2(spinner_radius + 5.0, spinner_radius);
                        
                        ui.painter().circle(
                            center,
                            spinner_radius,
                            egui::Color32::from_rgb(100, 100, 100),
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(220, 220, 220)),
                        );
                        
                        let spinner_point = center + egui::vec2(
                            spinner_radius * spinner_angle.cos(),
                            spinner_radius * spinner_angle.sin(),
                        );
                        
                        ui.painter().line_segment(
                            [center, spinner_point],
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 80, 80)),
                        );
                        
                        ui.add_space(spinner_radius * 2.0);
                        ui.label(egui::RichText::new("Calculating hash...").size(14.0));
                    });
                    
                    // Check calculation results
                    if let Some(rx) = &self.rx {
                        if let Ok(result) = rx.try_recv() {
                            self.hash_result = Some(result);
                            self.calculating = false;
                            self.rx = None;
                            self.tx = None;
                        }
                    }
                    
                    // macOS style cancel button
                    ui.vertical_centered(|ui| {
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Cancel")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(200, 60, 60))
                        ).min_size(egui::vec2(100.0, 28.0))).clicked() {
                            if let Some(tx) = &self.tx {
                                let _ = tx.send(());
                            }
                            self.calculating = false;
                            self.rx = None;
                            self.tx = None;
                        }
                    });
                }

                // Show result with fade-in animation
                if let Some(result) = &self.hash_result {
                    ui.add_space(20.0);
                    match result {
                        Ok(hash) => {
                            // macOS style container for the hash
                            egui::Frame::group(ui.style())
                                .fill(if ui.visuals().dark_mode {
                                    egui::Color32::from_rgb(45, 45, 45)
                                } else {
                                    egui::Color32::from_rgb(245, 245, 247)
                                })
                                .stroke(egui::Stroke::new(1.0, if ui.visuals().dark_mode {
                                    egui::Color32::from_rgb(100, 100, 100)
                                } else {
                                    egui::Color32::from_rgb(220, 220, 220)
                                }))
                                .rounding(egui::Rounding::same(8.0))
                                .shadow(egui::epaint::Shadow {
                                    extrusion: 2.0,
                                    color: egui::Color32::from_black_alpha(20),
                                })
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.heading(egui::RichText::new("SHA-256 Hash").size(18.0));
                                        ui.add_space(5.0);
                                        
                                        // Hash with improved formatting
                                        let mut hash_text = hash.clone();
                                        
                                        // First set the background color of the area
                                        let background_color = if ui.visuals().dark_mode {
                                            egui::Color32::from_rgb(30, 30, 30)
                                        } else {
                                            egui::Color32::from_rgb(235, 235, 235)
                                        };
                                        
                                        let text_color = if ui.visuals().dark_mode {
                                            egui::Color32::from_rgb(220, 220, 220)
                                        } else {
                                            egui::Color32::from_rgb(50, 50, 50)
                                        };
                                        
                                        // Create a frame with the desired background color
                                        egui::Frame::none()
                                            .fill(background_color)
                                            .inner_margin(egui::style::Margin::same(8.0))
                                            .show(ui, |ui| {
                                                ui.add(
                                                    egui::TextEdit::multiline(&mut hash_text.as_str())
                                                        .desired_width(ui.available_width())
                                                        .font(egui::TextStyle::Monospace)
                                                        .interactive(false)
                                                        .text_color(text_color)
                                                );
                                            });
                                        
                                        ui.add_space(5.0);
                                        
                                        // Button to copy to clipboard with hover effect
                                        if ui.add(egui::Button::new(
                                            egui::RichText::new("Copy to Clipboard")
                                                .size(14.0)
                                        ).min_size(egui::vec2(150.0, 30.0))).clicked() {
                                            ui.output_mut(|o| o.copied_text = hash.clone());
                                        }
                                    });
                                });
                        },
                        Err(error) => {
                            // macOS style error message
                            egui::Frame::group(ui.style())
                                .fill(egui::Color32::from_rgb(252, 235, 235))
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 150, 150)))
                                .rounding(egui::Rounding::same(8.0))
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(200, 60, 60),
                                            egui::RichText::new("Error").size(16.0).strong()
                                        );
                                        ui.label(
                                            egui::RichText::new(format!("{}", error))
                                                .color(egui::Color32::from_rgb(150, 60, 60))
                                        );
                                    });
                                });
                        }
                    }
                }
                
                // macOS style footer
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("HashSafe © 2025")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(150, 150, 150))
                    );
                });
            });
            
            // Request repaint for animations
            ctx.request_repaint();
        }
    }

    pub fn run_gui() -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(450.0, 580.0)),
            min_window_size: Some(egui::vec2(400.0, 500.0)),
            transparent: false,
            default_theme: Theme::Dark,  // Changed to dark theme by default
            follow_system_theme: true,   // Follow system configuration
            ..Default::default()
        };
        
        eframe::run_native(
            "HashSafe", 
            options,
            Box::new(|_cc: &CreationContext| Box::new(HashApp::default()))
        )
    }
}

fn main() {
    let args = Args::parse();

    // Determine whether to use the CLI or GUI interface
    if args.cli || args.file.is_some() {
        // CLI Mode
        if let Some(file_path) = args.file {
            if let Err(e) = run_cli(file_path) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        } else {
            eprintln!("In CLI mode, you must specify a file with --file");
            std::process::exit(1);
        }
    } else {
        // GUI Mode
        #[cfg(feature = "gui")]
        {
            if let Err(e) = gui::run_gui() {
                eprintln!("Error starting GUI: {}", e);
                std::process::exit(1);
            }
        }
        
        #[cfg(not(feature = "gui"))]
        {
            eprintln!("This version was compiled without GUI support.");
            eprintln!("Use --file to specify a file in CLI mode.");
            std::process::exit(1);
        }
    }
}