use crate::engine::runner::Runner;
use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Color32, Context, FontId, Label, Layout, RichText, ScrollArea, TopBottomPanel};

pub fn render_console(ctx: &Context, runner: &mut Runner, show_chart: &mut bool) {
    TopBottomPanel::bottom("bottom_console")
        .resizable(true)
        .default_height(140.0)
        .height_range(90.0..=380.0)
        .show(ctx, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("Konsol ve Çıktı Günlüğü")
                        .size(13.0)
                        .strong()
                        .color(Color32::from_rgb(138, 150, 164)),
                );
                ui.label(
                    RichText::new(format!("({} kayıt)", runner.logs.len()))
                        .size(11.0)
                        .weak(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .button("Temizle")
                        .on_hover_text("Konsol günlüğünü temizle")
                        .clicked()
                    {
                        runner.logs.clear();
                    }
                    if ui
                        .button("Kopyala")
                        .on_hover_text("Tüm konsol günlüğünü panoya kopyala")
                        .clicked()
                    {
                        let all_logs = runner.logs.join("\n");
                        ctx.copy_text(all_logs);
                    }
                    if ui
                        .button("📊 Grafik Göster")
                        .on_hover_text("Algoritma sonuç ve değişken grafiğini aç")
                        .clicked()
                    {
                        *show_chart = true;
                    }
                });
            });
            ui.separator();

            if runner.logs.is_empty() {
                ui.label(
                    RichText::new("Henüz çıktı kaydı yok. Çalıştır butonuna basın.").weak(),
                );
            } else {
                let mut job = LayoutJob::default();

                for (idx, log) in runner.logs.iter().enumerate() {
                    if idx > 0 {
                        job.append(
                            "\n",
                            0.0,
                            TextFormat {
                                font_id: FontId::monospace(12.0),
                                color: Color32::WHITE,
                                ..Default::default()
                            },
                        );
                    }

                    // Line number prefix
                    job.append(
                        &format!("{:3}  ", idx + 1),
                        0.0,
                        TextFormat {
                            font_id: FontId::monospace(11.0),
                            color: Color32::from_rgb(100, 116, 139),
                            ..Default::default()
                        },
                    );

                    // Syntax highlight colors per log type
                    let color = if log.contains("[Hata]")
                        || log.contains("[UYARI]")
                        || log.contains("Error")
                    {
                        Color32::from_rgb(248, 113, 113)
                    } else if log.contains("[Başlatıldı]")
                        || log.contains("[Tamamlandı]")
                        || log.contains("[Durduruldu]")
                    {
                        Color32::from_rgb(45, 212, 191)
                    } else if log.contains("[Giriş]") {
                        Color32::from_rgb(125, 211, 252)
                    } else if log.contains("[Çıktı]") {
                        Color32::from_rgb(251, 191, 36)
                    } else if log.contains("[KOŞUL]") {
                        Color32::from_rgb(244, 208, 111)
                    } else if log.contains("[FONKSİYON]")
                        || log.contains("[ÇAĞRI]")
                        || log.contains("[Geri Dönüldü]")
                    {
                        Color32::from_rgb(232, 121, 249)
                    } else if log.contains("[TOPLA]")
                        || log.contains("[ÇIKAR]")
                        || log.contains("[ÇARP]")
                        || log.contains("[BÖL]")
                    {
                        Color32::from_rgb(192, 132, 252)
                    } else if log.contains("[BİRLEŞTİR]") || log.contains("[KARŞILAŞTIR]") {
                        Color32::from_rgb(94, 234, 212)
                    } else {
                        Color32::from_rgb(226, 232, 240)
                    };

                    job.append(
                        log,
                        0.0,
                        TextFormat {
                            font_id: FontId::monospace(12.0),
                            color,
                            ..Default::default()
                        },
                    );
                }

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.add(Label::new(job).selectable(true));
                    });
            }
        });
}
