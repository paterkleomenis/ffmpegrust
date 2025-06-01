use crate::app::{FFmpegApp, ActiveTab};
use crate::conversion::ConversionMode;
use eframe::egui;

impl eframe::App for FFmpegApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_status();

        // Custom dark theme styling
        let mut style = (*ctx.style()).clone();
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.indent = 25.0;
        
        // Dark theme colors
        style.visuals.dark_mode = true;
        style.visuals.window_fill = egui::Color32::from_gray(20);
        style.visuals.panel_fill = egui::Color32::from_gray(25);
        style.visuals.faint_bg_color = egui::Color32::from_gray(30);
        style.visuals.extreme_bg_color = egui::Color32::from_gray(15);
        style.visuals.code_bg_color = egui::Color32::from_gray(35);
        
        ctx.set_style(style);
        ctx.set_visuals(egui::Visuals::dark());

        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::none().fill(egui::Color32::from_gray(15)).inner_margin(15.0))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(egui::RichText::new("🎬 FFmpeg Converter Pro").size(28.0).color(egui::Color32::WHITE).strong());
                    ui.label(egui::RichText::new("Professional Video Conversion & Remuxing Tool").size(14.0).color(egui::Color32::from_rgb(150, 150, 150)));
                });
            });

        egui::TopBottomPanel::bottom("controls")
            .frame(egui::Frame::none().fill(egui::Color32::from_gray(15)).inner_margin(15.0))
            .show(ctx, |ui| {
                self.show_main_controls(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            self.show_tabs(ui);
            ui.add_space(20.0);
            
            match self.active_tab {
                ActiveTab::Basic => self.show_basic_tab(ui),
                ActiveTab::Advanced => self.show_advanced_tab(ui),
                ActiveTab::Progress => self.show_progress_tab(ui),
            }
        });

        if self.is_converting {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

impl FFmpegApp {
    fn show_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.style_mut().spacing.button_padding = egui::vec2(25.0, 15.0);
            
            let basic_style = if self.active_tab == ActiveTab::Basic {
                egui::RichText::new("📁 Basic").size(16.0).color(egui::Color32::WHITE).strong()
            } else {
                egui::RichText::new("📁 Basic").size(16.0).color(egui::Color32::LIGHT_GRAY)
            };
            let basic_button = egui::SelectableLabel::new(self.active_tab == ActiveTab::Basic, basic_style);
            if ui.add(basic_button).clicked() {
                self.active_tab = ActiveTab::Basic;
            }

            ui.add_space(10.0);

            let advanced_style = if self.active_tab == ActiveTab::Advanced {
                egui::RichText::new("⚙️ Advanced").size(16.0).color(egui::Color32::WHITE).strong()
            } else {
                egui::RichText::new("⚙️ Advanced").size(16.0).color(egui::Color32::LIGHT_GRAY)
            };
            let advanced_button = egui::SelectableLabel::new(self.active_tab == ActiveTab::Advanced, advanced_style);
            if ui.add(advanced_button).clicked() {
                self.active_tab = ActiveTab::Advanced;
            }

            ui.add_space(10.0);

            let progress_style = if self.active_tab == ActiveTab::Progress {
                egui::RichText::new("📊 Progress").size(16.0).color(egui::Color32::WHITE).strong()
            } else {
                egui::RichText::new("📊 Progress").size(16.0).color(egui::Color32::LIGHT_GRAY)
            };
            let progress_button = egui::SelectableLabel::new(self.active_tab == ActiveTab::Progress, progress_style);
            if ui.add(progress_button).clicked() {
                self.active_tab = ActiveTab::Progress;
            }
        });
        
        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);
    }

    fn show_basic_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // File Selection Card
            egui::Frame::none()
                .fill(egui::Color32::from_gray(30))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(45)))
                .rounding(10.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(egui::RichText::new("📁 File Selection").color(egui::Color32::WHITE).size(18.0));
                        ui.add_space(12.0);
                        
                        self.show_file_selection(ui);
                    });
                });

            ui.add_space(15.0);

            // Quick Presets Card
            egui::Frame::none()
                .fill(egui::Color32::from_gray(30))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(45)))
                .rounding(10.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(egui::RichText::new("🎯 Quick Presets").color(egui::Color32::WHITE).size(18.0));
                        ui.add_space(12.0);
                        
                        self.show_presets(ui);
                    });
                });

            ui.add_space(15.0);

            // Basic Settings Card
            egui::Frame::none()
                .fill(egui::Color32::from_gray(30))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(45)))
                .rounding(10.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(egui::RichText::new("⚙️ Basic Settings").color(egui::Color32::WHITE).size(18.0));
                        ui.add_space(12.0);
                        
                        self.show_basic_settings(ui);
                    });
                });

            ui.add_space(15.0);

            // Status Card
            self.show_status_card(ui);
        });
    }

    fn show_advanced_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Advanced Conversion Settings Card
            egui::Frame::none()
                .fill(egui::Color32::from_gray(30))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(45)))
                .rounding(10.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(egui::RichText::new("🔧 Advanced Settings").color(egui::Color32::WHITE).size(18.0));
                        ui.add_space(12.0);
                        
                        self.show_advanced_settings(ui);
                    });
                });

            ui.add_space(15.0);

            // Hardware Acceleration Card
            egui::Frame::none()
                .fill(egui::Color32::from_gray(30))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(45)))
                .rounding(10.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(egui::RichText::new("🚀 Performance Options").color(egui::Color32::WHITE).size(18.0));
                        ui.add_space(12.0);
                        
                        self.show_performance_options(ui);
                    });
                });
        });
    }

    fn show_progress_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            if self.is_converting {
                self.show_detailed_progress(ui);
            } else {
                ui.add_space(50.0);
                ui.heading(egui::RichText::new("⏳ No Active Conversion").size(20.0).color(egui::Color32::GRAY));
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Start a conversion to see detailed progress information").color(egui::Color32::GRAY));
            }
        });
    }

    fn show_file_selection(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("file_selection")
            .num_columns(3)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Input File:").strong());
                ui.add_sized([350.0, 25.0], egui::TextEdit::singleline(&mut self.input_file).hint_text("Select input video file..."));
                if ui.add_sized([100.0, 25.0], egui::Button::new("📁 Browse")).clicked() {
                    self.select_input();
                }
                ui.end_row();

                ui.label(egui::RichText::new("Output File:").strong());
                ui.add_sized([350.0, 25.0], egui::TextEdit::singleline(&mut self.output_file).hint_text("Output file path..."));
                if ui.add_sized([100.0, 25.0], egui::Button::new("💾 Save As")).clicked() {
                    self.select_output();
                }
                ui.end_row();
            });
    }

    fn show_basic_settings(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("basic_settings")
            .num_columns(2)
            .spacing([20.0, 15.0])
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Mode:").strong());
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.mode, ConversionMode::Convert, "🔄 Convert");
                    ui.radio_value(&mut self.mode, ConversionMode::Remux, "📦 Remux");
                    
                    if self.mode == ConversionMode::Convert && self.smart_copy {
                        ui.label(egui::RichText::new("(Smart Copy Active)").color(egui::Color32::LIGHT_BLUE));
                    }
                });
                ui.end_row();

                ui.label(egui::RichText::new("Format:").strong());
                egui::ComboBox::from_id_source("container_basic")
                    .selected_text(format!("{} Container", self.container.to_uppercase()))
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        let old_container = self.container.clone();
                        ui.selectable_value(&mut self.container, "mp4".to_string(), "📺 MP4 - Most compatible");
                        ui.selectable_value(&mut self.container, "mkv".to_string(), "🎬 MKV - Supports all codecs");
                        ui.selectable_value(&mut self.container, "mov".to_string(), "🎥 MOV - QuickTime");
                        ui.selectable_value(&mut self.container, "avi".to_string(), "📼 AVI - Legacy format");
                        ui.selectable_value(&mut self.container, "webm".to_string(), "🌐 WebM - Web optimized");

                        if self.container != old_container {
                            self.update_output_extension();
                            self.update_config_from_current_settings();
                        }
                    });
                ui.end_row();

                if self.mode == ConversionMode::Convert && !self.smart_copy {
                    ui.label(egui::RichText::new("Quality:").strong());
                    ui.horizontal(|ui| {
                        let mut quality_val = self.quality.parse::<i32>().unwrap_or(23);
                        let old_quality = quality_val;
                        ui.add(egui::Slider::new(&mut quality_val, 18..=30).text("CRF"));
                        self.quality = quality_val.to_string();
                        if quality_val != old_quality {
                            self.update_config_from_current_settings();
                        }
                        
                        let (quality_desc, color) = match quality_val {
                            18..=20 => ("✨ Visually lossless", egui::Color32::GREEN),
                            21..=23 => ("🎯 High quality", egui::Color32::LIGHT_GREEN),
                            24..=26 => ("👌 Good quality", egui::Color32::YELLOW),
                            27..=28 => ("📱 Acceptable", egui::Color32::from_rgb(255, 165, 0)),
                            _ => ("⚠️ Low quality", egui::Color32::RED),
                        };
                        ui.label(egui::RichText::new(quality_desc).color(color));
                    });
                    ui.end_row();
                }
            });
    }

    fn show_advanced_settings(&mut self, ui: &mut egui::Ui) {
        if self.mode == ConversionMode::Convert {
            egui::Grid::new("advanced_settings")
                .num_columns(2)
                .spacing([20.0, 15.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Video Codec:").strong());
                    egui::ComboBox::from_id_source("video_codec_advanced")
                        .selected_text(&self.video_codec)
                        .width(250.0)
                        .show_ui(ui, |ui| {
                            let old_codec = self.video_codec.clone();
                            ui.selectable_value(&mut self.video_codec, "libx264".to_string(), "🎬 H.264 - Most compatible");
                            ui.selectable_value(&mut self.video_codec, "libx265".to_string(), "🔥 H.265 - Better compression");
                            ui.selectable_value(&mut self.video_codec, "libvpx".to_string(), "🌐 VP8 - Web codec");
                            ui.selectable_value(&mut self.video_codec, "libvpx-vp9".to_string(), "⚡ VP9 - Advanced web");
                            ui.selectable_value(&mut self.video_codec, "libaom-av1".to_string(), "🚀 AV1 - Next generation");
                            if self.video_codec != old_codec {
                                self.update_config_from_current_settings();
                            }
                        });
                    ui.end_row();

                    ui.label(egui::RichText::new("Audio Codec:").strong());
                    egui::ComboBox::from_id_source("audio_codec_advanced")
                        .selected_text(&self.audio_codec)
                        .width(250.0)
                        .show_ui(ui, |ui| {
                            let old_codec = self.audio_codec.clone();
                            ui.selectable_value(&mut self.audio_codec, "aac".to_string(), "🎵 AAC - High quality");
                            ui.selectable_value(&mut self.audio_codec, "mp3".to_string(), "🎶 MP3 - Universal");
                            ui.selectable_value(&mut self.audio_codec, "libopus".to_string(), "🎙️ Opus - Efficient");
                            ui.selectable_value(&mut self.audio_codec, "libvorbis".to_string(), "🔊 Vorbis - Open source");
                            ui.selectable_value(&mut self.audio_codec, "flac".to_string(), "💎 FLAC - Lossless");
                            ui.selectable_value(&mut self.audio_codec, "pcm_s16le".to_string(), "📡 PCM 16-bit - Uncompressed");
                            ui.selectable_value(&mut self.audio_codec, "pcm_s24le".to_string(), "🎚️ PCM 24-bit - Professional");
                            if self.audio_codec != old_codec {
                                self.update_config_from_current_settings();
                            }
                        });
                    ui.end_row();

                    if !self.smart_copy {
                        ui.label(egui::RichText::new("Quality Mode:").strong());
                        ui.horizontal(|ui| {
                            let mut quality_val = self.quality.parse::<i32>().unwrap_or(23);
                            let old_quality = quality_val;
                            ui.add(egui::Slider::new(&mut quality_val, 15..=35).text("CRF Value"));
                            self.quality = quality_val.to_string();
                            if quality_val != old_quality {
                                self.update_config_from_current_settings();
                            }
                        });
                        ui.end_row();
                    }
                });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(egui::RichText::new("📦 Remux Mode").size(18.0).color(egui::Color32::LIGHT_BLUE));
                ui.add_space(10.0);
                ui.label("No additional settings available in remux mode.");
                ui.label("All streams will be copied without re-encoding.");
            });
        }
    }

    fn show_performance_options(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("performance_options")
            .num_columns(2)
            .spacing([20.0, 15.0])
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Hardware Acceleration:").strong());
                ui.horizontal(|ui| {
                    let old_hw_accel = self.use_hardware_accel;
                    ui.checkbox(&mut self.use_hardware_accel, "🚀 Enable GPU acceleration");
                    if self.use_hardware_accel != old_hw_accel {
                        self.update_config_from_current_settings();
                    }
                });
                ui.end_row();

                ui.label(egui::RichText::new("Smart Copy Mode:").strong());
                ui.horizontal(|ui| {
                    let old_smart_copy = self.smart_copy;
                    ui.checkbox(&mut self.smart_copy, "🧠 Enable smart copy");
                    if self.smart_copy != old_smart_copy {
                        self.update_config_from_current_settings();
                    }
                });
                ui.end_row();
            });

        ui.add_space(10.0);
        
        if self.smart_copy {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(0, 100, 200, 30))
                .rounding(5.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("💡 Smart Copy Mode").color(egui::Color32::LIGHT_BLUE).strong());
                    ui.label("• Video stream: Copied without re-encoding (fast)");
                    ui.label("• Audio stream: Converted to high-quality PCM");
                    ui.label("• WebM files: Video converted to ProRes for compatibility");
                });
        }

        if self.use_hardware_accel {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(0, 200, 100, 30))
                .rounding(5.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("🚀 Hardware Acceleration").color(egui::Color32::LIGHT_GREEN).strong());
                    ui.label("• Automatic detection of available GPU encoders");
                    ui.label("• Significantly faster encoding speeds");
                    ui.label("• Supports NVIDIA NVENC, Intel QSV, and more");
                });
        }
    }

    fn show_presets(&mut self, ui: &mut egui::Ui) {
        let preset_style = ui.style_mut();
        preset_style.spacing.button_padding = egui::vec2(15.0, 10.0);

        egui::Grid::new("presets")
            .num_columns(3)
            .spacing([15.0, 15.0])
            .show(ui, |ui| {
                if ui.add_sized([180.0, 50.0], 
                    egui::Button::new(egui::RichText::new("🌐 Web Optimized").size(14.0))
                    .fill(egui::Color32::from_rgb(50, 100, 200))
                ).clicked() {
                    self.apply_preset("Web (H.264/MP4)");
                }
                if ui.add_sized([180.0, 50.0], 
                    egui::Button::new(egui::RichText::new("💎 High Quality").size(14.0))
                    .fill(egui::Color32::from_rgb(100, 50, 200))
                ).clicked() {
                    self.apply_preset("High Quality (H.265/MKV)");
                }
                if ui.add_sized([180.0, 50.0], 
                    egui::Button::new(egui::RichText::new("📱 Small File").size(14.0))
                    .fill(egui::Color32::from_rgb(200, 100, 50))
                ).clicked() {
                    self.apply_preset("Small File (H.265)");
                }
                ui.end_row();

                if ui.add_sized([180.0, 50.0], 
                    egui::Button::new(egui::RichText::new("⚡ Fast Remux").size(14.0))
                    .fill(egui::Color32::from_rgb(50, 200, 100))
                ).clicked() {
                    self.apply_preset("Fast Remux");
                }
                if ui.add_sized([180.0, 50.0], 
                    egui::Button::new(egui::RichText::new("🎵 Pro Audio").size(14.0))
                    .fill(egui::Color32::from_rgb(200, 50, 100))
                ).clicked() {
                    self.apply_preset("MOV PCM (Pro Audio)");
                }
                ui.end_row();
            });
    }

    fn show_main_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            
            let start_enabled = !self.is_converting && !self.input_file.is_empty() && !self.output_file.is_empty();
            let start_button = egui::Button::new(
                egui::RichText::new("🚀 Start Conversion").size(16.0)
            ).min_size(egui::vec2(180.0, 45.0));
            
            if ui.add_enabled(start_enabled, start_button).clicked() {
                self.start_conversion();
            }

            ui.add_space(20.0);

            let stop_button = egui::Button::new(
                egui::RichText::new("⏹ Stop Conversion").size(16.0)
            ).min_size(egui::vec2(180.0, 45.0));
            
            if ui.add_enabled(self.is_converting, stop_button).clicked() {
                self.stop_conversion();
            }

            ui.add_space(20.0);

            let clear_button = egui::Button::new(
                egui::RichText::new("🗑 Clear All").size(16.0)
            ).min_size(egui::vec2(120.0, 45.0));
            
            if ui.add(clear_button).clicked() {
                *self = Self::new();
            }

            ui.add_space(10.0);
        });
    }

    fn show_status_card(&mut self, ui: &mut egui::Ui) {
        if let Some(error) = &self.error {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(200, 50, 50, 50))
                .rounding(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("❌ Error").color(egui::Color32::LIGHT_RED));
                    });
                    ui.separator();
                    ui.label(egui::RichText::new(error).color(egui::Color32::LIGHT_RED));
                });
        } else if !self.input_file.is_empty() && !self.output_file.is_empty() && !self.is_converting {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(50, 200, 50, 50))
                .rounding(8.0)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("✅ Ready to Convert").color(egui::Color32::LIGHT_GREEN));
                    });
                    ui.separator();
                    
                    egui::Grid::new("ready_stats")
                        .num_columns(2)
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Mode:").strong());
                            let mode_display = if self.mode == ConversionMode::Convert && self.smart_copy {
                                "Smart Copy (Remux-like)"
                            } else {
                                match self.mode {
                                    ConversionMode::Convert => "Convert",
                                    ConversionMode::Remux => "Remux",
                                }
                            };
                            ui.label(mode_display);
                            ui.end_row();
                            
                            if self.mode == ConversionMode::Convert {
                                if self.smart_copy {
                                    ui.label(egui::RichText::new("Operation:").strong());
                                    ui.label("Fast copy + audio conversion");
                                    ui.end_row();
                                    
                                    ui.label(egui::RichText::new("Video:").strong());
                                    ui.label("Copy (no re-encoding)");
                                    ui.end_row();
                                    
                                    ui.label(egui::RichText::new("Audio:").strong());
                                    ui.label("Convert to PCM 16-bit");
                                    ui.end_row();
                                } else {
                                    ui.label(egui::RichText::new("Video:").strong());
                                    ui.label(format!("{} (CRF {})", self.video_codec, self.quality));
                                    ui.end_row();
                                    
                                    ui.label(egui::RichText::new("Audio:").strong());
                                    ui.label(format!("{}", self.audio_codec));
                                    ui.end_row();
                                }
                            } else {
                                ui.label(egui::RichText::new("Operation:").strong());
                                ui.label("Copy all streams (no re-encoding)");
                                ui.end_row();
                            }
                            
                            ui.label(egui::RichText::new("Output:").strong());
                            ui.label(format!("{}", self.container.to_uppercase()));
                            ui.end_row();
                        });
                });
        }
    }

    fn show_detailed_progress(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(egui::Color32::from_gray(30))
            .stroke(egui::Stroke::new(2.0, egui::Color32::from_gray(50)))
            .rounding(15.0)
            .inner_margin(25.0)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(egui::RichText::new("🔄 Converting").size(24.0).color(egui::Color32::WHITE));
                    ui.add_space(15.0);
                });

                // Progress bar
                let progress_fraction = self.progress.percentage / 100.0;
                let progress_bar = egui::ProgressBar::new(progress_fraction)
                    .text(format!("{:.1}%", self.progress.percentage))
                    .desired_width(500.0)
                    .desired_height(30.0);
                ui.add(progress_bar);

                ui.add_space(20.0);

                // Progress stats in cards
                ui.columns(3, |columns| {
                    columns[0].vertical_centered(|ui| {
                        egui::Frame::none()
                            .fill(egui::Color32::from_gray(35))
                            .rounding(8.0)
                            .inner_margin(15.0)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("📊 Frames").strong().size(14.0));
                                    ui.label(egui::RichText::new(format!("{}", self.progress.current_frame)).size(20.0).color(egui::Color32::LIGHT_BLUE));
                                });
                            });
                        
                        ui.add_space(10.0);
                        
                        egui::Frame::none()
                            .fill(egui::Color32::from_gray(35))
                            .rounding(8.0)
                            .inner_margin(15.0)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("🎯 FPS").strong().size(14.0));
                                    ui.label(egui::RichText::new(format!("{:.1}", self.progress.fps)).size(20.0).color(egui::Color32::LIGHT_GREEN));
                                });
                            });
                    });

                    columns[1].vertical_centered(|ui| {
                        egui::Frame::none()
                            .fill(egui::Color32::from_gray(35))
                            .rounding(8.0)
                            .inner_margin(15.0)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("⚡ Speed").strong().size(14.0));
                                    ui.label(egui::RichText::new(format!("{:.2}x", self.progress.speed)).size(20.0).color(egui::Color32::YELLOW));
                                });
                            });
                        
                        ui.add_space(10.0);
                        
                        egui::Frame::none()
                            .fill(egui::Color32::from_gray(35))
                            .rounding(8.0)
                            .inner_margin(15.0)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("⏱ Time").strong().size(14.0));
                                    ui.label(egui::RichText::new(&self.progress.time_elapsed).size(16.0).color(egui::Color32::WHITE));
                                });
                            });
                    });

                    columns[2].vertical_centered(|ui| {
                        if !self.progress.size.is_empty() {
                            egui::Frame::none()
                                .fill(egui::Color32::from_gray(35))
                                .rounding(8.0)
                                .inner_margin(15.0)
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(egui::RichText::new("💾 Size").strong().size(14.0));
                                        ui.label(egui::RichText::new(&self.progress.size).size(16.0).color(egui::Color32::LIGHT_GRAY));
                                    });
                                });
                            
                            ui.add_space(10.0);
                        }
                        
                        if let Some(eta) = self.progress.eta {
                            egui::Frame::none()
                                .fill(egui::Color32::from_gray(35))
                                .rounding(8.0)
                                .inner_margin(15.0)
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(egui::RichText::new("⏳ ETA").strong().size(14.0));
                                        ui.label(egui::RichText::new(format!("{}s", eta.as_secs())).size(16.0).color(egui::Color32::from_rgb(255, 165, 0)));
                                    });
                                });
                        }
                    });
                });

                if !self.progress.bitrate.is_empty() {
                    ui.add_space(15.0);
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new(format!("📈 Bitrate: {}", self.progress.bitrate)).size(14.0).color(egui::Color32::LIGHT_GRAY));
                    });
                }
            });
    }
}