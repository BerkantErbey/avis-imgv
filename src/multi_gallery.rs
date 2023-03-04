use crate::{
    config::{ContextMenuEntry, MultiGalleryConfig},
    thumbnail_image::ThumbnailImage,
    user_action::build_context_menu,
};
use eframe::{
    egui::{self, Ui},
    epaint::Vec2,
};
use std::path::PathBuf;

pub struct MultiGallery {
    imgs: Vec<ThumbnailImage>,
    config: MultiGalleryConfig,
    selected_image_name: Option<String>,
    prev_img_size: f32,
    prev_scroll_offset: f32,
    total_rows: usize,
    images_per_row: usize,
    prev_images_per_row: usize,
    prev_row_range_start: usize,
}

impl MultiGallery {
    pub fn new(
        image_paths: &Vec<PathBuf>,
        config: MultiGalleryConfig,
        output_profile: &String,
    ) -> MultiGallery {
        let imgs = ThumbnailImage::from_paths(image_paths, output_profile);
        let mut mg = MultiGallery {
            total_rows: Self::calc_total_rows(imgs.len(), config.images_per_row),
            imgs,
            selected_image_name: None,
            images_per_row: config.images_per_row,
            prev_images_per_row: config.images_per_row,
            config,
            prev_img_size: 0.,
            prev_scroll_offset: 0.,
            prev_row_range_start: 0,
        };

        mg.imgs.sort_by(|a, b| a.name.cmp(&b.name));

        mg
    }

    pub fn ui(&mut self, ctx: &egui::Context, jump_to_index: &mut Option<usize>) {
        self.handle_input(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0., 0.);
            ui.set_min_width(ui.available_width());

            let mut loading_imgs = self.imgs.iter().filter(|i| i.is_loading()).count();
            let img_size = ui.available_width() / self.images_per_row as f32;

            let mut scroll_area = egui::ScrollArea::vertical().drag_to_scroll(true);

            //Since image size changes when we resize the window, we need to compensate the scroll
            //offset as show_rows assumes fixed widget sizes
            if img_size != self.prev_img_size {
                scroll_area = scroll_area.scroll_offset(Vec2 {
                    x: 0.,
                    y: img_size * self.prev_scroll_offset / self.prev_img_size,
                });
            }

            if self.images_per_row != self.prev_images_per_row {
                let target_row =
                    (self.prev_row_range_start * self.prev_images_per_row) / self.images_per_row;

                scroll_area = scroll_area.scroll_offset(Vec2 {
                    x: 0.,
                    y: img_size * target_row as f32,
                });
            }

            match jump_to_index.take() {
                Some(mut i) => {
                    //Get start of the row index so it's easier to calculate the offset
                    i = i - (i % self.images_per_row);
                    let scroll_offset = ((i as f32) / self.images_per_row as f32) * img_size;
                    scroll_area = scroll_area.scroll_offset(Vec2 {
                        x: 0.,
                        y: scroll_offset,
                    })
                }
                None => {}
            };

            let scroll_area_response =
                scroll_area.show_rows(ui, img_size, self.total_rows, |ui, row_range| {
                    ui.spacing_mut().item_spacing = Vec2::new(0., 0.);

                    let preload_from = if row_range.start <= self.config.preloaded_rows {
                        0
                    } else {
                        row_range.start - self.config.preloaded_rows
                    };

                    let preload_to = if row_range.end + self.config.preloaded_rows > self.total_rows
                    {
                        self.total_rows
                    } else {
                        row_range.end + self.config.preloaded_rows
                    };

                    //first we go over the visible ones
                    for r in row_range.start..row_range.end {
                        for i in r * self.images_per_row..(r + 1) * self.images_per_row {
                            self.load_unload_image(
                                i,
                                row_range.start,
                                row_range.end,
                                &mut loading_imgs,
                                img_size,
                            );
                        }
                    }

                    //then in the down direction as the user is most likely to scroll down
                    for r in row_range.end..self.total_rows {
                        for i in r * self.images_per_row..(r + 1) * self.images_per_row {
                            self.load_unload_image(
                                i,
                                preload_from,
                                preload_to,
                                &mut loading_imgs,
                                img_size,
                            );
                        }
                    }

                    //then up
                    for r in 0..row_range.start {
                        for i in r * self.images_per_row..(r + 1) * self.images_per_row {
                            self.load_unload_image(
                                i,
                                preload_from,
                                preload_to,
                                &mut loading_imgs,
                                img_size,
                            );
                        }
                    }

                    for r in row_range.clone() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(0., 0.);
                            for j in r * self.images_per_row..(r + 1) * self.images_per_row {
                                match &mut self.imgs.get_mut(j) {
                                    Some(img) => Self::show_image(
                                        img,
                                        ui,
                                        ctx,
                                        img_size,
                                        &mut self.selected_image_name,
                                        &self.config.margin_size,
                                        &self.config.context_menu,
                                    ),
                                    None => {}
                                }
                            }
                        });
                    }

                    if ui.input_mut(|i| i.consume_shortcut(&self.config.sc_scroll.kbd_shortcut)) {
                        ui.scroll_with_delta(Vec2::new(0., (img_size * 0.5) * -1.));
                    }

                    self.prev_row_range_start = row_range.start;
                });

            self.prev_scroll_offset = scroll_area_response.state.offset.y;
            self.prev_img_size = img_size;
            self.prev_images_per_row = self.images_per_row;
        });
    }

    fn load_unload_image(
        &mut self,
        i: usize,
        preload_from: usize,
        preload_to: usize,
        loading_imgs: &mut usize,
        image_size: f32,
    ) {
        let img = &mut match self.imgs.get_mut(i) {
            Some(img) => img,
            None => return,
        };

        if i >= preload_from * self.images_per_row && i <= preload_to * self.images_per_row {
            if loading_imgs != &self.config.simultaneous_load {
                //Double the square size so we have a little downscale going on
                //Looks better than without and won't impact speed much. Possibly add as a config
                if img.load((image_size * 2.) as u32) {
                    *loading_imgs += 1;
                }
            }
        } else {
            img.unload_delayed();
            img.unload(i);
        }
    }

    fn show_image(
        image: &mut ThumbnailImage,
        ui: &mut Ui,
        ctx: &egui::Context,
        max_size: f32,
        select_image_name: &mut Option<String>,
        margin_size: &f32,
        context_menu: &Vec<ContextMenuEntry>,
    ) {
        match image.ui(ui, [max_size, max_size], margin_size) {
            Some(resp) => {
                if resp.clicked() {
                    *select_image_name = Some(image.name.clone());
                }
                if resp.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                build_context_menu(context_menu, resp, image.path.clone())
            }
            None => {}
        };
    }

    pub fn handle_input(&mut self, ui: &egui::Context) {
        if (ui.input_mut(|i| i.consume_shortcut(&self.config.sc_more_per_row.kbd_shortcut))
            || ui.input(|i| i.zoom_delta() < 1.))
            && self.images_per_row <= 15
        {
            self.images_per_row += 1;
            self.total_rows = Self::calc_total_rows(self.imgs.len(), self.images_per_row);
        }

        if (ui.input_mut(|i| i.consume_shortcut(&self.config.sc_less_per_row.kbd_shortcut))
            || ui.input(|i| i.zoom_delta() > 1.))
            && self.images_per_row != 1
        {
            self.images_per_row -= 1;
            self.total_rows = Self::calc_total_rows(self.imgs.len(), self.images_per_row);
        }
    }

    pub fn selected_image_name(&mut self) -> Option<String> {
        //We want it to be consumed
        self.selected_image_name.take()
    }

    pub fn calc_total_rows(imgs_len: usize, imgs_per_row: usize) -> usize {
        //div_ceil will be available in the next release. Avoids conversions..
        (imgs_len as f32 / imgs_per_row as f32).ceil() as usize
    }
}
