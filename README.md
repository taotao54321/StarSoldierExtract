# NES Star Soldier data extractor

## Usage

### extract enemy bytecode

```sh
mkdir output/
cargo run --bin bytecode -- StarSoldier.nes output/
```

### extract ground cell matrix

```sh
cargo run --bin cell_matrix -- StarSoldier.nes CellMatrix-1.png
cargo run --bin cell_matrix -- --second-round StarSoldier.nes CellMatrix-2.png
```

### extract ground map

```sh
# 2nd round, stage 16
cargo run --bin ground -- StarSoldier.nes --second-round 16 Ground-2-16.png
```

### extract meta sprites

```sh
mkdir output/
cargo run --bin meta_sprite -- StarSoldier.nes output/
```

### extract meta sprite matrix

```sh
cargo run --bin meta_sprite_matrix -- StarSoldier.nes MetaSpriteMatrix-1.png
cargo run --bin meta_sprite_matrix -- --second-round StarSoldier.nes MetaSpriteMatrix-2.png
```

### extract musics as [FlMML](https://github.com/argentum384/flmml-on-html5)

```sh
mkdir output/
cargo run --bin music -- StarSoldier.nes output/
```
