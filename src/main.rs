use image::EncodableLayout;
use lab::Lab;
use rumqttc::{AsyncClient, MqttOptions};

fn get_average_color(frame: &[u8]) -> [u8; 3] {
    let colors = lab::rgb_bytes_to_labs(frame.as_bytes());

    let mut summed_colors = colors.iter().fold(
        Lab {
            l: 0.,
            a: 0.,
            b: 0.,
        },
        |mut acc, pixel| {
            acc.l += pixel.l;
            acc.a += pixel.a;
            acc.b += pixel.b;
            acc
        },
    );

    let average_color = {
        let pixel_count = colors.len() as f32;
        summed_colors.l /= pixel_count;
        summed_colors.a /= pixel_count;
        summed_colors.b /= pixel_count;
        summed_colors
    };

    let average_rgb = average_color.to_rgb();
    average_rgb
}

fn get_color_stream() -> impl Iterator<Item = [u8; 3]> {
    camera_capture::create(0)
        .unwrap()
        .fps(5.0)
        .unwrap()
        .start()
        .unwrap()
        .map(|frame| get_average_color(frame.as_bytes()))
}

#[tokio::main]
pub async fn main() -> Result<(), ()> {
    let mut mqttoptions = MqttOptions::new("bias-lighting", "htpc.lan", 1883);
    mqttoptions.set_keep_alive(5);

    let (client, mut event_loop) = AsyncClient::new(mqttoptions, 10);

    tokio::spawn(async move {
        loop {
            event_loop.poll().await.unwrap();
        }
    });

    for average_rgb in get_color_stream() {
        let hex_color = format!("#{}", hex::encode(average_rgb));
        println!("{}", hex_color);
        client
            .publish(
                "zigbee2mqtt/led/set/color",
                rumqttc::QoS::AtMostOnce,
                false,
                hex_color,
            )
            .await
            .unwrap();
    }

    Ok(())
}
