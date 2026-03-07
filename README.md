# Seikaku Engine

EVE Online 舰船配置属性计算引擎。输入 pb2 格式的游戏数据文件和舰船配置，输出所有 Dogma 属性的完整计算结果。

## 计算原理

引擎采用多阶段计算流程：

- [第一阶段](./src/calculate/pass_1.rs)：收集舰体、模块、植入体、增效剂的全部 Dogma 属性基础值。
- [第二阶段](./src/calculate/pass_2.rs)：收集舰体、模块、植入体、增效剂的全部 Dogma 效果，并按修改器类型分发到目标物品的属性上。
- [第三阶段](./src/calculate/pass_3.rs)：将所有 Dogma 效果应用到舰体/模块/植入体，计算最终属性值（含堆叠惩罚）。
- [第四阶段](./src/calculate/pass_4.rs)：补充计算一些 Dogma 本身无法直接表达的复合属性（如电容稳定性、DPS 等）。

> 第四阶段新增的属性 ID 均为负值，以便与游戏内原生属性区分。

---

## 数据文件

引擎读取来自 [EVEShipFit/data](https://github.com/EVEShipFit/data) 的 protobuf 格式数据文件。需要以下四个 `.pb2` 文件，放在同一目录下：

| 文件名                | 内容                         |
| --------------------- | ---------------------------- |
| `dogmaAttributes.pb2` | 全部 Dogma 属性定义          |
| `dogmaEffects.pb2`    | 全部 Dogma 效果定义          |
| `typeDogma.pb2`       | 各 type 对应的属性/效果列表  |
| `types.pb2`           | 物品基础信息（名称、分类等） |

目录路径在初始化引擎时传入。

---

## 开发环境准备

确保已安装 [Rust](https://www.rust-lang.org/tools/install)。

本项目**不需要** npm，protoc 编译器已通过 `protoc-bin-vendored` 自动捆绑，无需手动安装任何额外工具，直接 `cargo build` 即可。

---

## 集成方式

### Flutter（跨平台，推荐）

通过 `dart:ffi` 调用 Rust 编译出的动态/静态库，实现跨平台无服务端计算。

**1. 编译库文件**

```bash
# 当前平台（调试）
cargo build --no-default-features --features flutter,eft

# 当前平台（发布）
cargo build --release --no-default-features --features flutter,eft

# 交叉编译示例（Android ARM64）
cargo build --release --target aarch64-linux-android --no-default-features --features flutter,eft

# 交叉编译示例（iOS）
cargo build --release --target aarch64-apple-ios --no-default-features --features flutter,eft
```

编译产物在 `target/<profile>/` 中：

- Windows：`seikaku_engine.dll`
- Linux/Android：`libseikaku_engine.so`
- macOS/iOS：`libseikaku_engine.a`（staticlib）

**2. 暴露的 C 接口**（见 [`src/flutter/mod.rs`](./src/flutter/mod.rs)）

| 函数                                               | 说明                                           |
| -------------------------------------------------- | ---------------------------------------------- |
| `seikaku_init(path)`                               | 从指定目录加载所有 pb2 文件，返回引擎句柄      |
| `seikaku_calculate(engine, fit_json, skills_json)` | 计算配置属性，返回 JSON 字符串                 |
| `seikaku_load_eft(engine, eft_text)`               | 解析 EFT 格式文本，返回配置 JSON               |
| `seikaku_free_string(ptr)`                         | 释放引擎返回的字符串（必须调用，防止内存泄漏） |
| `seikaku_free(engine)`                             | 释放引擎句柄（必须调用）                       |

**3. Dart 调用示例**

```dart
import 'dart:ffi';
import 'dart:convert';
import 'package:ffi/ffi.dart';

// 加载库
final lib = DynamicLibrary.open('libseikaku_engine.so'); // Android / Linux
// final lib = DynamicLibrary.open('seikaku_engine.dll'); // Windows
// DynamicLibrary.process() 用于 iOS

// 绑定函数
final _init = lib.lookupFunction<
    Pointer Function(Pointer<Utf8>),
    Pointer Function(Pointer<Utf8>)>('seikaku_init');

final _calculate = lib.lookupFunction<
    Pointer<Utf8> Function(Pointer, Pointer<Utf8>, Pointer<Utf8>),
    Pointer<Utf8> Function(Pointer, Pointer<Utf8>, Pointer<Utf8>)>('seikaku_calculate');

final _freeString = lib.lookupFunction<
    Void Function(Pointer<Utf8>),
    void Function(Pointer<Utf8>)>('seikaku_free_string');

final _free = lib.lookupFunction<
    Void Function(Pointer),
    void Function(Pointer)>('seikaku_free');

// 初始化（传入 pb2 文件所在目录路径）
final pathPtr = '/data/user/0/com.example.app/files/evedata'.toNativeUtf8();
final engine = _init(pathPtr);
malloc.free(pathPtr);

// 构造配置
final fit = jsonEncode({
  'ship_type_id': 24690,
  'modules': [
    {
      'type_id': 2048,
      'slot': {'type': 'High', 'index': 0},
      'state': 'Active',
      'charge': null
    }
  ],
  'drones': []
});
final skills = jsonEncode({'3300': 5, '3301': 4});

// 计算并读取结果
final fitPtr = fit.toNativeUtf8();
final skillsPtr = skills.toNativeUtf8();
final resultPtr = _calculate(engine, fitPtr, skillsPtr);
final stats = jsonDecode(resultPtr.toDartString());

// 释放内存
_freeString(resultPtr);
malloc.free(fitPtr);
malloc.free(skillsPtr);

// 不再需要引擎时
_free(engine);
```

**配置 JSON 格式（`EsfFit`）**

```json
{
  "ship_type_id": 24690,
  "modules": [
    {
      "type_id": 2048,
      "slot": { "type": "High", "index": 0 },
      "state": "Active",
      "charge": { "type_id": 192 }
    }
  ],
  "drones": [{ "type_id": 2185, "state": "Active" }],
  "implants": [
    { "type_id": 9899, "index": 1 },
    { "type_id": 9941, "index": 2 }
  ],
  "boosters": [
    { "type_id": 28672, "index": 1 }
  ]
}
```

- `state` 可选值：`Passive` | `Online` | `Active` | `Overload`
- `slot.type` 可选值：`High` | `Medium` | `Low` | `Rig` | `SubSystem` | `Service`
- `charge` 可为 `null`
- `implants` / `boosters` 字段可省略（默认为空数组）；`index` 为插槽编号（植入体 1–10）

**技能 JSON 格式**

```json
{ "3300": 5, "3301": 4 }
```

键为技能 type ID（字符串），值为等级 05。未列出的技能默认视为 L1。

---

### 命令行工具（CLI）

保留了用于调试的 CLI 工具。

**编译**

```bash
cargo build --release --no-default-features --features rust,eft
```

**用法**

```bash
# 从 EFT 文件读取配置并计算（-p 指定 pb2 目录）
./target/release/seikaku-cli -p ./data -e fit.eft

# 从 stdin 读取 EFT
cat fit.eft | ./target/release/seikaku-cli -p ./data

# 指定模块状态（24 字符：前8高槽 中8中槽 后8低槽）
# P=离线  O=在线  A=激活  V=过载
./target/release/seikaku-cli -p ./data -e fit.eft --state "AAAAAAAAAAAAAAAAAAAAAAAAAA"

# 指定技能文件
./target/release/seikaku-cli -p ./data -e fit.eft -f skills.json
```

输出为 JSON，包含电容、攻击、防御、机动、定向等完整统计数据。

---

## 项目结构

```
src/
 lib.rs              # 库入口，模块声明
 main.rs             # CLI 入口（rust feature）
 data_types.rs       # 核心数据结构定义
 info.rs             # Info trait（数据访问接口）
 calculate/          # 四阶段计算逻辑
    mod.rs
    item.rs
    pass_1.rs ~ pass_4.rs
 eft/                # EFT 格式解析（eft feature）
    mod.rs
 flutter/            # Flutter FFI C 接口（flutter feature）
    mod.rs
 rust/               # protobuf 数据加载（rust/flutter feature）
     mod.rs
     info.rs
     protobuf.rs
```

## Cargo Feature 说明

| Feature   | 说明                                   | 默认启用 |
| --------- | -------------------------------------- | :------: |
| `flutter` | Flutter FFI C 接口 + protobuf 数据加载 |          |
| `eft`     | EFT 格式解析支持                       |          |
| `rust`    | CLI 工具（依赖 `clap`）                |          |
