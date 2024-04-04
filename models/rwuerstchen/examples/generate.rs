use futures_util::StreamExt;
use rwuerstchen::*;

#[tokio::main]
async fn main() {
    let model = Wuerstchen::builder().build().await.unwrap();
    let settings = WuerstchenInferenceSettings::new(
        "a cute cat with a hat in a room covered with fur with incredible detail",
    )
    .with_n_steps(2);

    if let Ok(mut images) = model.run(settings) {
        while let Some(image) = images.next().await {
            if let Some(buf) = image.generated_image() {
                buf.save(&format!("{}.png", image.sample_num())).unwrap();
            }
        }
    }
}
