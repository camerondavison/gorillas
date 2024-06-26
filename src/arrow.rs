use crate::prelude::*;

#[derive(Bundle)]
pub(crate) struct ArrowBundle {
    shape: ShapeBundle,
    fill: Fill,
    stroke: Stroke,
}

pub(crate) fn build_arrow_shape(
    outline_color: Color,
    fill_color: Color,
    raw_length: i16,
    height: u16,
    x: f32,
    y: f32,
    z: f32,
) -> ArrowBundle {
    let length = raw_length.abs() as u16;
    let scale = if raw_length < 0 { -1.0 } else { 1.0 };
    let svg_path_string = arrow_path(&length, height);
    ArrowBundle {
        shape: ShapeBundle {
            path: GeometryBuilder::build_as(&shapes::SvgPathShape {
                svg_doc_size_in_px: Vec2::new(length as f32, height as f32),
                svg_path_string,
            }),
            spatial: SpatialBundle::from_transform(
                Transform::from_translation(Vec3::new(x, y, z))
                    .with_scale(Vec2::new(scale, 1.0).extend(1.0)),
            ),
            ..default()
        },
        fill: Fill::color(fill_color),
        stroke: Stroke::color(outline_color),
    }
}

fn arrow_path(length: &u16, height: u16) -> String {
    let mut svg_path_string = format!("M {} {}", length / 2, height / 2);
    svg_path_string.push_str(&format!(
        "h {} v -6 l 8 8 l -8 8 v -6 h -{} v -4",
        length, length
    ));
    svg_path_string
}
