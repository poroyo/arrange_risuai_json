use eframe::egui;
use egui_dnd::dnd;
use std::sync::mpsc;
use test_app::{
    console_error, console_log, file_read, Data, HasherbleValue, LorebookJson, RegexJson,
};

const PADDING_WIDE: f32 = 10.;
const LEFT_WIDTH: f32 = 300.;

pub struct BigFrame {
    file_json: Option<Data>,
    value: Option<Vec<serde_json::Value>>,
    value_hash: Vec<HasherbleValue>,
    contents_receiver: Option<mpsc::Receiver<Vec<u8>>>,
    content: Option<String>,
    is_lore: bool,
    lore_ver: Option<u8>,
    selected_index: Option<usize>,
    is_new_entry: bool,
    new_entry_title: String,
    is_submitted: bool,
}


impl BigFrame {
    pub fn _new(cc: &eframe::CreationContext<'_>) -> Self {
        _setup_custom_font(&cc.egui_ctx);
        Self {
            file_json: None,
            contents_receiver: None,
            content: None,
            value: None,
            value_hash: Vec::new(),
            is_lore: false,
            lore_ver: None,
            selected_index: None,
            is_new_entry: false,
            new_entry_title: String::from(""),
            is_submitted: false,
        }
    }

    fn render_leftside(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("RisuAI JSON Arrange")
                        .text_style(egui::TextStyle::Heading),
                );
                let button = ui.add_sized(egui::vec2(80., 30.), egui::Button::new("Import"));
                if button.clicked() {
                    let (file_sender, file_receiver) = mpsc::channel();
                    self.contents_receiver = Some(file_receiver);

                    let task = rfd::AsyncFileDialog::new()
                        .add_filter("json", &["json"])
                        .pick_file();

                    wasm_bindgen_futures::spawn_local(async move {
                        let file = task.await;
                        if let Some(file) = file {
                            let file_contents = file.read().await;
                            if let Err(error) = file_sender.send(file_contents) {
                                eprintln!("An error occurred during file communication: {error}")
                            }
                        }
                    })
                }
            });

            if self.contents_receiver.is_some() {
                ctx.request_repaint();
            }

            self.update_channel();

            ui.with_layout(egui::Layout::top_down_justified(egui::Align::RIGHT), |ui| {
                if self.file_json.is_some() {
                    self.is_lore = self.file_json.as_ref().unwrap().is_lore;
                    if self.is_lore {
                        self.lore_ver = self.file_json.as_ref().unwrap().lore_ver;
                    }

                    self.value = Some(self.file_json.take().unwrap().data);

                    let haserbler = self
                        .value
                        .take()
                        .unwrap()
                        .into_iter()
                        .enumerate()
                        .map(|(index, element)| HasherbleValue(element, index))
                        .collect::<Vec<_>>();

                    self.value_hash = haserbler;
                }
                if !self.value_hash.is_empty() {
                    if self.is_lore {
                        ui.label(
                            egui::RichText::new("Lorebook").text_style(egui::TextStyle::Heading),
                        );
                    } else {
                        ui.label(egui::RichText::new("RegEx").text_style(egui::TextStyle::Heading));
                    }

                    ui.horizontal(|ui| {   
                        let button = ui.add_sized(egui::vec2(80., 30.), egui::Button::new("Export"));
                        if button.clicked() {
                            let json = to_string_json(self);
                            if let Ok(json) = json {
                                let task = rfd::AsyncFileDialog::new()
                                    .save_file();
    
                                wasm_bindgen_futures::spawn_local(async move {
                                    let file = task.await;
                                    if let Some(file) = file {
                                        if let Err(error) = file.write(json.as_bytes()).await {
                                            console_error(&format!(
                                                "An error occurred while writing the file: {error}"
                                            ));
                                        }
                                    }
                                });
                            } else {
                                console_error("An error occurred while converting the file.");
                            }
                        }
                        if !self.is_lore {
                            let new_entry_button = ui.add_sized(egui::vec2(80., 30.), egui::Button::new("New Entry"));
                            if new_entry_button.clicked() {
                                self.is_new_entry = true;
                            }
    
                            self.add_empty_entry(ctx);
                        }

                    });
                }
            });
        });

        ui.add_space(PADDING_WIDE);
        ui.separator();

        dnd_ui(ui, self);
        self.deletd_selected_entry();
    }

    fn render_rightside(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(content) = &self.content {
                ui.label(content);
            }
        });
    }
}

impl BigFrame {
    fn update_channel(&mut self) {
        if let Some(rx) = &self.contents_receiver {
            match rx.try_recv() {
                Ok(file_string) => {

                    match file_read(file_string) {
                        Ok(file_string) => {
                            console_log(&format!("{:?}", file_string));
                            self.file_json = Some(file_string);
                            console_log("The data in the file was opened.");
                        }
                        Err(error) => {
                            console_error(&format!("Invalid file: {:?}", error));
                        }
                    }
                }
                Err(error) => match error {
                    mpsc::TryRecvError::Empty => {}
                    mpsc::TryRecvError::Disconnected => {
                        self.contents_receiver.take();
                    }
                },
            }
        }
    }

    fn add_empty_entry(&mut self, ctx: &egui::Context) {
        if self.is_new_entry {
            egui::Window::new("Input entry title").open(&mut self.is_new_entry).show(ctx, |ui| {
                ui.text_edit_singleline(&mut self.new_entry_title);

                if ui.button("Submit").clicked() {
                    self.is_submitted = true;
                }
            });
        }

        if self.is_submitted {
            let new_title_ref = self.new_entry_title.as_str();
            let empty_value = serde_json::json!({
                "comment": new_title_ref,
                "in": "",
                "out": "",
                "type": "editdisplay",
                "ableFlag": false
            });
            let hasherble = HasherbleValue(empty_value, self.value_hash.len());
            self.value_hash.push(hasherble);

            self.new_entry_title = String::new();
            self.is_new_entry = false;
            self.is_submitted = false;
        }
    }

    fn deletd_selected_entry(&mut self) {
        if let Some(index) = self.selected_index {
            self.value_hash.remove(index);
            self.selected_index.take();
        }
    }
}

impl eframe::App for BigFrame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .exact_width(LEFT_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                self.render_leftside(ui, ctx);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_rightside(ui, ctx);
        });
    }
}

fn _setup_custom_font(ctx: &egui::Context) {
    // start default fonts
    let mut fonts = egui::FontDefinitions::default();

    // install font
    fonts.font_data.insert(
        "NanumGothic".to_string(),
        egui::FontData::from_static(include_bytes!("../font/NanumGothic.ttf")),
    );

    // put my font first, proportional
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "NanumGothic".to_string());
    // set font
    ctx.set_fonts(fonts);
}

fn dnd_ui(ui: &mut egui::Ui, big_frame: &mut BigFrame) {
    let item_size = egui::vec2(LEFT_WIDTH, 30.);
    let is_lore = if big_frame.is_lore { "content" } else { "out" };
    // let mut source = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        let response = dnd(ui, "dnd_json").show_custom(|ui, iter| {
            big_frame
                .value_hash
                .iter_mut()
                .enumerate()
                .for_each(|(index, item)| {
                    iter.next(
                        ui,
                        egui::Id::new(item.1),
                        index,
                        |ui, item_handle| {
                            item_handle.ui_sized(ui, item_size, |ui, handle, _state| {
                                ui.horizontal_wrapped(|ui| {
                                    handle.ui_sized(ui, item_size, |ui| {
                                        let lable = item.0["comment"].as_str().unwrap();
                                        let response =
                                            ui.add_sized(item_size, egui::Button::new(lable));

                                        if response.clicked() {
                                            if big_frame.is_lore {
                                                big_frame.content = Some(
                                                    item.0[is_lore].as_str().unwrap().to_owned(),
                                                );
                                            } else {
                                                big_frame.content = Some(item.0.to_string());
                                            }
                                        }
                                        
                                        response.context_menu(|ui| {
                                            if ui.button("Delete").clicked() {
                                                big_frame.selected_index = Some(index);
                                                ui.close_menu();
                                            }
                                        });

                                    });
                                });
                            })
                        },
                    );
                });
        });

        response.update_vec(&mut big_frame.value_hash);

        if let Some(reason) = response.cancellation_reason() {
            console_error(&format!("Drag has been cancelled because of {:?}", reason));
        }
    });
}

fn to_string_json(big_frame: &BigFrame) -> Result<String, serde_json::Error> {
    let vec = &big_frame.value_hash;
    let is_lore = big_frame.is_lore;
    let lore_ver = big_frame.lore_ver;

    let vec = vec
        .into_iter()
        .map(|element| element.0.clone())
        .collect::<Vec<_>>();

    if is_lore {
        let new_json = LorebookJson {
            app_type: "risu".to_string(),
            ver: lore_ver.unwrap(),
            data: vec,
        };
        let new_json = serde_json::to_string(&new_json);
        new_json
    } else {
        let new_json = RegexJson {
            app_type: "regex".to_string(),
            data: vec,
        };
        let new_json = serde_json::to_string(&new_json);
        new_json
    }
}

