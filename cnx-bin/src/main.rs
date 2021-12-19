use anyhow::Result;
use byte_unit::ByteUnit;
use cnx::text::*;
use cnx::widgets::*;
use cnx::{Cnx, Position};
use cnx_contrib::widgets::battery::*;
use cnx_contrib::widgets::disk_usage::*;
use cnx_contrib::widgets::*;
use weathernoaa::weather::WeatherInfo;

fn pango_markup_render(color: Color, start_text: String, text: String) -> String {
    format!(
            "<span foreground=\"#808080\">[</span>{} <span foreground=\"{}\">{}</span><span foreground=\"#808080\">]</span>",
        start_text, color.to_hex(), text
        )
}

fn pango_markup_single_render(color: Color, start_text: String) -> String {
    format!(
            "<span foreground=\"#808080\">[</span>{}<span foreground=\"{}\"></span><span foreground=\"#808080\">]</span>",
        start_text, color.to_hex()
        )
}

fn weather_sky_condition(condition: String) -> &'static str {
    match &condition[..] {
        "clear" => "🌣",
        "sunny" => "🌣",
        "mostly clear" => "🌤",
        "mostly sunny" => "🌤",
        "partly sunny" => "⛅",
        "fair" => "🌑",
        "cloudy" => "☁",
        "overcast" => "☁",
        "partly cloudy" => "⛅",
        "mostly cloudy" => "🌧",
        "considerable cloudines" => "☔",
        _ => "🌑",
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let attr = Attributes {
        font: Font::new("Ubuntu Mono Bold 14"),
        fg_color: Color::white(),
        bg_color: None,
        padding: Padding::new(0.0, 0.0, 0.0, 0.0),
    };

    let pager_attr = Attributes {
        font: Font::new("Ubuntu Mono Bold 14"),
        fg_color: Color::white(),
        bg_color: Some(Color::blue()),
        padding: Padding::new(8.0, 8.0, 0.0, 0.0),
    };

    // let sensors = Sensors::new(attr.clone(), vec!["Core 0", "Core 1"]);
    let battery_render = |battery_info: BatteryInfo| {
        let percentage = battery_info.capacity;

        let default_text = format!("🔋{percentage:.0}%", percentage = percentage,);
        pango_markup_single_render(Color::white(), default_text)
    };

    let battery = Battery::new_with_render(attr.clone(), Color::red(), None, battery_render);
    let render = |load| {
        let mut color = Color::yellow().to_hex();
        if load < 5 {
            color = Color::green().to_hex();
        }
        if load > 50 {
            color = Color::red().to_hex();
        }
        format!(
            "<span foreground=\"#808080\">[</span>Cpu: <span foreground=\"{}\">{}%</span><span foreground=\"#808080\">]</span>",
            color, load
        )
    };
    let cpu = cpu::Cpu::new_with_render(attr.clone(), render)?;

    let volume = volume::Volume::new(attr.clone());

    let default_threshold = Threshold::default();

    let wireless =
        wireless::Wireless::new(attr.clone(), "wlp2s0".to_owned(), Some(default_threshold));

    let disk_render = |disk_info: DiskInfo| {
        let used = disk_info.used.get_adjusted_unit(ByteUnit::GiB).format(0);
        let total = disk_info.total.get_adjusted_unit(ByteUnit::GiB).format(0);
        let disk_text = format!("🏠 {}/{}", used, total);
        pango_markup_single_render(Color::white(), disk_text)
    };

    let disk_usage = disk_usage::DiskUsage::new_with_render(attr.clone(), "/home".into(), disk_render);

    let weather_render = |weather: WeatherInfo| {
        let sky_condition = weather_sky_condition(weather.sky_condition);
        let weather_text = format!("BLR: {} :", sky_condition);
        let weather_temp = format!(" {}°C", weather.temperature.celsius);
        pango_markup_render(Color::white(), weather_text, weather_temp)
    };

    let weather = weather::Weather::new_with_render(attr.clone(), "VOBL".into(), weather_render);

    let mut p2_attr = pager_attr.clone();
    p2_attr.bg_color = None;

    let time_template = Some("<span foreground=\"#808080\">[</span>%d-%m-%Y %a %I:%M %p<span foreground=\"#808080\">]</span>".into());

    Cnx::new(Position::Bottom)?
        .add_widget(Pager::new(pager_attr, p2_attr))?
        .add_widget(ActiveWindowTitle::new(attr.clone()))?
        .add_widget(cpu)?
        .add_widget(weather)?
        .add_widget(disk_usage)?
        .add_widget(wireless)?
        .add_widget(volume)?
        .add_widget(battery)?
        .add_widget(Clock::new(attr, time_template))?
        .run()
        .await?;

    Ok(())
}
