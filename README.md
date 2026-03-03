# Seikaku Engine

EVE Online 舰船配装属性计算引擎。

输入为 EVE SDE SQLite 数据库与一套舰船配装，输出为完整的 Dogma 属性计算结果（JSON 格式）。

## 计算原理

引擎采用多阶段 Pass 方式计算：

- [Pass 1](./src/calculate/pass_1.rs)：收集船体及模块的所有 Dogma 属性基础值
- [Pass 2](./src/calculate/pass_2.rs)：收集船体及模块的所有 Dogma 效果
- [Pass 3](./src/calculate/pass_3.rs)：将效果应用于属性，计算最终属性值
- [Pass 4](./src/calculate/pass_4.rs)：计算 Dogma 无法直接表达的复合属性（DPS、EHP 等）

## 扩展属性（Pass 4）

Pass 4 会生成游戏内不存在、但计算复杂的属性，以方便上层渲染使用。  
这些属性的 ID 恒为**负值**，以便与标准属性区分。

## 数据来源

本引擎使用 [EVE SDE SQLite](https://github.com/garveen/eve-sde-converter) 数据库作为唯一数据源，无需任何 Protobuf 或 npm 依赖。  
数据库表结构参见 [SDE_DATABASE.md](./SDE_DATABASE.md)。

## 开发环境

安装 [Rust 工具链](https://www.rust-lang.org/tools/install) 后即可直接构建，无需其他外部依赖。

### 构建动态链接库（cdylib）

```bash
cargo build --release --lib --no-default-features --features rust,eft
```

输出文件：
| 平台 | 文件名 |
|---|---|
| Windows | `seikaku_engine.dll` |
| macOS | `libseikaku_engine.dylib` |
| Linux | `libseikaku_engine.so` |
| iOS | `libseikaku_engine.a` |

### 构建命令行工具（CLI）

```bash
cargo build --release --no-default-features --features cli,eft
```

**CLI 用法示例：**

```bash
# 从 EFT 文件读取配装并计算（结果为 JSON）
seikaku-engine -d path/to/sde.sqlite -e fit.eft

# 通过标准输入传入 EFT 字符串
cat fit.eft | seikaku-engine -d path/to/sde.sqlite

# 指定技能文件（JSON 格式，key 为技能 typeID，value 为等级）
seikaku-engine -d path/to/sde.sqlite -e fit.eft -f skills.json
```

## C FFI 跨语言调用

动态库导出了标准 C 接口，可在 C/C++、Python、Java、C# 等语言中调用。

### 接口定义

```c
// 初始化引擎，传入 SDE SQLite 数据库路径
// 返回引擎句柄，失败返回 NULL
void* seikaku_init(const char* sqlite_path);

// 通过 EFT 格式字符串计算舰船属性
// skills_json: {"typeID": level, ...}，可传 NULL 使用默认技能
// 返回 JSON 字符串，需调用 seikaku_free_string 释放
char* seikaku_calculate_eft(const void* engine, const char* eft_str, const char* skills_json);

// 通过 EsfFit JSON 字符串计算舰船属性
// 返回 JSON 字符串，需调用 seikaku_free_string 释放
char* seikaku_calculate(const void* engine, const char* fit_json, const char* skills_json);

// 释放引擎句柄
void seikaku_free(void* engine);

// 释放由本库分配的字符串
void seikaku_free_string(char* s);
```

### C 调用示例

```c
#include <stdio.h>

int main() {
    void* engine = seikaku_init("sde.sqlite");
    if (!engine) return 1;

    const char* eft = "[Rifter, My Fit]\nSmall ACM Compact Armor Repairer\n";
    char* result = seikaku_calculate_eft(engine, eft, NULL);
    if (result) {
        printf("%s\n", result);
        seikaku_free_string(result);
    }

    seikaku_free(engine);
    return 0;
}
```

### Python 调用示例

```python
import ctypes, json

lib = ctypes.CDLL("seikaku_engine.dll")  # 或 .so / .dylib
lib.seikaku_init.restype = ctypes.c_void_p
lib.seikaku_calculate_eft.restype = ctypes.c_char_p
lib.seikaku_free.argtypes = [ctypes.c_void_p]
lib.seikaku_free_string.argtypes = [ctypes.c_char_p]

engine = lib.seikaku_init(b"sde.sqlite")
result = lib.seikaku_calculate_eft(engine, eft_bytes, None)
data = json.loads(result)
lib.seikaku_free_string(result)
lib.seikaku_free(engine)
```

## CI / Release

推送 `v*` 格式的 Tag（如 `v1.0.0`）后，GitHub Actions 将自动：

1. 为以下平台编译动态库
2. 打包并创建 GitHub Release

| 平台 | 构建目标 |
|---|---|
| Windows x86_64 | `x86_64-pc-windows-msvc` |
| macOS Universal | `x86_64-apple-darwin` + `aarch64-apple-darwin`（lipo 合并） |
| iOS aarch64 | `aarch64-apple-ios`（静态库） |
| Android arm64-v8a | `aarch64-linux-android` |
| Android armeabi-v7a | `armv7-linux-androideabi` |
| Android x86_64 | `x86_64-linux-android` |

## 配装 JSON 结构（EsfFit）

```jsonc
{
  "ship_type_id": 587,
  "modules": [
    {
      "type_id": 2048,
      "slot": { "type": "High", "index": 0 },
      "state": "Active",
      "charge": { "type_id": 212 }   // 可选
    }
  ],
  "drones": [
    { "type_id": 2488, "state": "Active" }
  ]
}
```

`state` 可选值：`Passive` / `Online` / `Active` / `Overload`  
`slot.type` 可选值：`High` / `Medium` / `Low` / `Rig` / `SubSystem` / `Service`
