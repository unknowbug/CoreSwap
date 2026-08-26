// buildnode_test.rs — 验证构建链贯通：JSON → build_node → DensityFunction → sample
use WorldgenRust::json::parse;
use WorldgenRust::density_builder::build_node;
use WorldgenRust::density::NoisePos;

fn main() {
    // 简单树：add(noise(continentalness POC), 0.5)
    let json = r#"{"type":"minecraft:add",
        "argument1":{"type":"minecraft:noise"},
        "argument2":{"type":"minecraft:constant","value":0.5}}"#;
    let v = parse(json).unwrap();
    let df = build_node(&v).unwrap();
    for (x, y, z) in [(0, 0, 0), (8, 64, 8), (100, -64, -40)] {
        println!("df({},{},{}) = {:.12}", x, y, z, df.sample(&NoisePos { x, y, z }));
    }
    println!("min={:.12} max={:.12}", df.min_value(), df.max_value());
}
