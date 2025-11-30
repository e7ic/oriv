# Slint UI 框架完整教程 - 从入门到精通

## 目录
1. [Slint 基础概念](#1-slint-基础概念)
2. [文件结构与组件](#2-文件结构与组件)
3. [属性系统](#3-属性系统)
4. [表达式与语句](#4-表达式与语句)
5. [布局与定位](#5-布局与定位)
6. [全局单例](#6-全局单例)
7. [重复与数据模型](#7-重复与数据模型)
8. [动画系统](#8-动画系统)
9. [状态管理](#9-状态管理)
10. [函数与回调](#10-函数与回调)
11. [名称解析与作用域](#11-名称解析与作用域)
12. [结构体与枚举](#12-结构体与枚举)
13. [常见陷阱与最佳实践](#13-常见陷阱与最佳实践)

---

## 1. Slint 基础概念

### 1.1 什么是 Slint?

Slint 是一个现代的声明式 UI 框架,用于创建跨平台的用户界面。它使用自己的 `.slint` 文件格式来定义界面,类似于 QML 或 SwiftUI。

**类比说明**: 如果你熟悉 HTML/CSS,可以把 Slint 想象成一个更强大的 HTML,它不仅可以定义界面结构,还能直接处理交互逻辑和动画。

### 1.2 核心概念

- **元素 (Elements)**: 界面的基本构建块,如 `Rectangle`、`Text`、`Image` 等
- **组件 (Components)**: 由多个元素组成的可复用单元
- **属性 (Properties)**: 元素的特征,如颜色、大小、位置等
- **响应式**: 属性之间可以建立依赖关系,当一个属性改变时,依赖它的属性自动更新

---

## 2. 文件结构与组件

### 2.1 基本文件结构

每个 `.slint` 文件定义一个或多个组件。文件必须以组件声明开始。

```slint
// 基本组件定义
component MyButton inherits Text {
    color: black;
    text: "Click Me";
}

// 导出组件(可被其他文件使用)
export component MyApp inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    
    MyButton {
        x: 10px;
        y: 10px;
    }
}
```

**关键概念**:
- 使用 `component` 关键字定义组件
- 使用 `inherits` 继承已有元素或组件
- 使用 `export` 导出组件供外部使用

### 2.2 元素命名

```slint
export component Example inherits Window {
    // 使用 := 为元素命名
    my_button := Rectangle {
        background: blue;
        width: 100px;
        height: 50px;
    }
    
    // 预定义名称
    // root - 根元素(Example)
    // self - 当前元素
    // parent - 父元素
    
    Text {
        text: "Width: " + root.my_button.width;
    }
}
```

**坑点警告**: 
- 不能重定义 `root`、`self`、`parent` 这些保留名称
- 元素命名使用 `:=` 而非 `=`

### 2.3 注释

```slint
// 单行注释

/*
   多行注释
   可以跨越多行
*/

component Button {
    // 属性注释
    background: blue; // 行尾注释
}
```

### 2.4 标识符规则

```slint
// 合法的标识符
component MyButton {}
component my_button {}
component my-button {}  // 推荐使用短横线

// foo_bar 和 foo-bar 是等价的(下划线会被规范化为短横线)
property <int> foo_bar: 10;
property <int> foo-bar: 10;  // 与上面相同

// 非法的标识符
// component 123button {}  // 不能以数字开头
// component -button {}     // 不能以短横线开头
```

**最佳实践**: 使用短横线分隔的小写字母命名(kebab-case)

### 2.5 条件元素

```slint
export component Example inherits Window {
    preferred-width: 200px;
    preferred-height: 100px;
    
    in-out property <bool> show-warning: false;
    
    // 条件渲染:只在 show-warning 为 true 时显示
    if show-warning : warning-text := Text {
        text: "⚠️ Warning!";
        color: red;
    }
    
    // 反向条件
    if !show-warning : normal-text := Text {
        text: "Everything is OK";
        color: green;
    }
    
    TouchArea {
        clicked => {
            show-warning = !show-warning;
        }
    }
}
```

**类比说明**: 类似于 React 的条件渲染或 Vue 的 `v-if`

**坑点警告**: 
- 条件元素只能在编译时或运行时创建/销毁,不会保留状态
- 频繁切换条件元素的性能比修改可见性属性要低

### 2.6 模块系统

#### 导出组件

```slint
// button.slint
component ButtonHelper inherits Rectangle {
    background: gray;
}

component Button inherits Rectangle {
    width: 100px;
    height: 40px;
    ButtonHelper { }
}

// 方式1: 导出列表
export { Button }

// 方式2: 重命名导出
export { Button as MyButton }

// 方式3: 直接导出
export component DirectButton inherits Rectangle { }
```

#### 导入组件

```slint
// app.slint
import { Button } from "./button.slint";
import { Button as CoolButton } from "./other_button.slint";

export component App inherits Window {
    Button { }        // 从 button.slint
    CoolButton { }    // 从 other_button.slint
}
```

#### 批量导入导出

```slint
// 导入多个
import { Button, Switch, Slider } from "widgets.slint";

// 重导出
export { Button, Switch } from "widgets.slint";

// 导出所有(每个文件只能用一次)
export * from "widgets.slint";
```

### 2.7 组件库

对于跨项目的组件库,使用 `@` 语法:

```slint
import { MyButton } from "@mylib/button.slint";
import { ThemeColors } from "@mylib";
```

**配置方法**:

**Rust (build.rs)**:
```rust
slint_build::compile_with_config(
    "ui/app.slint",
    slint_build::CompilerConfiguration::new()
        .with_library_paths(HashMap::from([
            ("mylib".into(), PathBuf::from("path/to/mylib"))
        ]))
).unwrap();
```

**VS Code (settings.json)**:
```json
{
    "slint.libraryPaths": {
        "mylib": "/path/to/mylib",
        "otherlib": "/path/to/otherlib/index.slint"
    }
}
```

**命令行**:
```bash
slint-viewer -Lmylib=/path/to/mylib app.slint
```

---

## 3. 属性系统

### 3.1 属性的基本使用

```slint
export component Example inherits Window {
    // 简单赋值(以分号结尾)
    width: 400px;
    
    // 代码块赋值(不需要分号)
    height: {
        400px
    }
    
    // 表达式绑定
    background: touch-area.pressed ? red : blue;
    
    // 等价的代码块形式
    background: {
        if touch-area.pressed {
            return red;
        } else {
            return blue;
        }
    }
    
    touch-area := TouchArea { }
}
```

**类比说明**: 
- 简单赋值类似于 CSS 的属性设置
- 表达式绑定类似于 Excel 的公式,自动响应依赖变化

### 3.2 声明自定义属性

```slint
export component Counter inherits Rectangle {
    // 声明一个 int 类型属性
    property <int> count;
    
    // 带默认值的属性
    property <int> max-count: 10;
    
    // 带初始表达式
    property <string> status: count >= max-count ? "Full" : "OK";
}
```

**属性类型**:
- `int` - 整数
- `float` - 浮点数
- `bool` - 布尔值
- `string` - 字符串
- `color` - 颜色
- `length` - 长度(带单位)
- `duration` - 时长
- 自定义结构体和枚举

### 3.3 属性可见性修饰符

```slint
export component DataWidget inherits Rectangle {
    // private (默认): 只能在组件内部访问
    private property <int> internal-counter: 0;
    
    // in: 输入属性,外部可以设置,组件内部不应修改
    in property <string> label: "Default";
    
    // out: 输出属性,只能由组件设置,外部只读
    out property <bool> is-ready: internal-counter > 5;
    
    // in-out: 双向属性,内外都可以读写
    in-out property <int> value;
}

export component App inherits Window {
    my-widget := DataWidget {
        label: "My Label";    // OK: 设置 in 属性
        value: 42;            // OK: 设置 in-out 属性
        // is-ready: true;    // 错误!不能设置 out 属性
    }
    
    Text {
        text: "Ready: " + my-widget.is-ready;  // OK: 读取 out 属性
    }
}
```

**最佳实践**:
- 用 `in` 表示配置选项(如颜色、文本)
- 用 `out` 表示状态输出(如是否按下、当前值)
- 用 `in-out` 表示需要双向绑定的值
- 用 `private` 保护内部实现细节

### 3.4 属性变化回调

```slint
import { LineEdit } from "std-widgets.slint";

export component Example inherits Window {
    VerticalLayout {
        input := LineEdit {
            // 当 text 属性改变时触发
            changed text => {
                output.text = "You typed: " + self.text;
            }
        }
        
        output := Text { }
    }
}
```

**重要特性**:
- 回调不是立即执行,而是在下一个事件循环中执行
- 同一事件循环中多次改变只触发一次回调
- 如果值改变后又恢复原值,回调不会执行

**⚠️ 危险警告 - 避免循环依赖**:

```slint
// ❌ 错误示例:创建无限循环
export component Bad inherits Rectangle {
    in-out property <int> foo;
    property <int> bar: foo + 1;
    
    // 危险!会创建循环
    changed bar => {
        foo += 1;  // 改变 foo -> bar 改变 -> 再次触发回调
    }
}

// ✅ 正确做法:使用绑定
export component Good inherits Rectangle {
    in-out property <int> foo;
    property <int> bar: foo + 1;  // 直接绑定,无需回调
}
```

**最佳实践**:
1. **优先使用声明式绑定**而非 `changed` 回调
2. 只在确实需要副作用(如调用外部 API)时使用回调
3. 避免在 `changed` 回调中修改可能影响触发属性的值

---

## 4. 表达式与语句

### 4.1 算术表达式

```slint
export component Example inherits Rectangle {
    property <int> result: 1 * 2 + 3 * 4;  // 结果: 14 (遵循运算优先级)
    
    // 支持的运算符
    property <int> add: 10 + 5;        // 加法: 15
    property <int> sub: 10 - 5;        // 减法: 5
    property <int> mul: 10 * 5;        // 乘法: 50
    property <int> div: 10 / 5;        // 除法: 2
    
    // 长度运算
    width: 100px + 50px;               // 150px
    height: parent.height / 2;         // 父元素高度的一半
}
```

### 4.2 字符串操作

```slint
export component Example {
    property <string> first-name: "John";
    property <string> last-name: "Doe";
    property <string> full-name: first-name + " " + last-name;  // "John Doe"
}
```

### 4.3 逻辑表达式

```slint
export component Example inherits Rectangle {
    in-out property <bool> is-enabled: true;
    in-out property <bool> is-visible: true;
    
    // 逻辑与
    property <bool> can-interact: is-enabled && is-visible;
    
    // 逻辑或
    property <bool> should-show: is-enabled || is-visible;
    
    // 逻辑非
    property <bool> is-disabled: !is-enabled;
}
```

### 4.4 比较运算符

```slint
export component Example {
    property <int> value: 42;
    
    property <bool> equal: value == 42;       // 相等
    property <bool> not-equal: value != 0;    // 不等
    property <bool> greater: value > 10;      // 大于
    property <bool> less: value < 100;        // 小于
    property <bool> gte: value >= 42;         // 大于等于
    property <bool> lte: value <= 42;         // 小于等于
}
```

### 4.5 三元运算符

```slint
export component Example inherits Window {
    preferred-width: 200px;
    preferred-height: 100px;
    
    touch := TouchArea { }
    
    // 条件 ? 真值 : 假值
    background: touch.pressed ? #111 : #eee;
    
    // 嵌套三元运算符
    border-color: !touch.enabled ? #888 : 
                  touch.pressed ? #aaa : 
                  #555;
}
```

**类比说明**: 类似于 JavaScript/C 的三元运算符

### 4.6 属性访问

```slint
export component Example inherits Rectangle {
    my-rect := Rectangle {
        x: 100px;
        y: 50px;
        width: 200px;
    }
    
    // 访问子元素属性
    property <length> rect-x: my-rect.x;
    property <length> rect-width: my-rect.width;
    
    // 链式访问
    text-element := Text {
        text: "Hello";
    }
    property <int> text-length: text-element.text.length;  // 字符串长度(如果支持)
}
```

### 4.7 语句

#### Let 语句(局部变量)

```slint
export component Example inherits Rectangle {
    callback calculate-sum(int, int) -> int;
    
    calculate-sum(a, b) => {
        // 声明局部变量(不可变)
        let sum = a + b;
        let doubled = sum * 2;
        
        // 带类型注解
        let result: int = doubled + 10;
        
        debug("Result:", result);
        return result;
    }
}
```

**特性**:
- 局部变量不可变(immutable)
- 不能重新声明同名变量
- 可选的类型注解

**坑点**: 局部变量不能在不同作用域重复声明,即使看起来不冲突:

```slint
// ❌ 错误
clicked => {
    let x = 1;
    if some-condition {
        let x = 2;  // 错误!不能重新声明 x
    }
}
```

#### 赋值语句

```slint
export component Example {
    in-out property <int> counter: 0;
    
    callback increment;
    callback reset;
    
    increment => {
        counter = counter + 1;  // 普通赋值
    }
    
    reset => {
        counter = 0;
    }
}
```

#### 自增赋值

```slint
export component Example {
    in-out property <int> value: 10;
    
    callback update;
    
    update => {
        value += 5;   // value = value + 5
        value -= 2;   // value = value - 2
        value *= 3;   // value = value * 3
        value /= 2;   // value = value / 2
    }
}
```

#### 条件语句

```slint
export component Example {
    in-out property <int> score: 0;
    out property <string> grade;
    
    callback calculate-grade;
    
    calculate-grade => {
        if score >= 90 {
            grade = "A";
        } else if score >= 80 {
            grade = "B";
        } else if score >= 70 {
            grade = "C";
        } else {
            grade = "F";
        }
    }
}
```

#### 调用回调

```slint
export component Example {
    callback my-callback(int) -> string;
    callback trigger;
    
    my-callback(x) => {
        return "Value: " + x;
    }
    
    trigger => {
        // 调用其他回调
        let result = root.my-callback(42);
        debug(result);
    }
}
```

---

## 5. 布局与定位

### 5.1 显式定位

```slint
export component Example inherits Window {
    width: 400px;
    height: 300px;
    
    // 绝对定位
    blue-rect := Rectangle {
        x: 50px;
        y: 50px;
        width: 200px;
        height: 100px;
        background: blue;
        
        // 相对父元素定位
        green-rect := Rectangle {
            x: 10px;  // 相对于 blue-rect
            y: 10px;
            width: 50px;
            height: 30px;
            background: green;
        }
    }
    
    // 使用表达式的响应式定位
    red-rect := Rectangle {
        x: 0;
        y: 0;
        width: parent.width - 100px;   // 窗口宽度 - 100px
        height: parent.height / 2;      // 窗口高度的一半
        background: red;
    }
}
```

**单位**:
- `px` - 逻辑像素(推荐,自动处理高DPI)
- `phx` - 物理像素
- `%` - 百分比(相对父元素)

```slint
export component Example inherits Rectangle {
    width: 200px;   // 200逻辑像素
    height: 50%;    // 父元素高度的50%
    
    child := Rectangle {
        width: 100%;    // 父元素(Example)宽度的100%
    }
}
```

**默认值**:
- `x` 和 `y` 默认居中元素
- `width` 和 `height` 取决于元素类型:
  - `Image`、`Text`、Widget: 根据内容自动计算
  - `Rectangle`、`TouchArea`、`FocusScope`: 默认填充父元素
  - 布局元素: 默认填充父元素

### 5.2 首选尺寸

```slint
export component MyComponent inherits Rectangle {
    // 设置首选尺寸
    preferred-width: 300px;
    preferred-height: 200px;
    
    // 使用父元素尺寸作为首选尺寸
    Rectangle {
        preferred-width: 100%;   // 默认使用父元素宽度
        preferred-height: 100%;
        background: blue;
    }
}
```

**类比说明**: `preferred-*` 类似于 CSS 的 `min-content` 或 Flexbox 的 `flex-basis`

### 5.3 垂直和水平布局

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    
    // 水平布局
    HorizontalLayout {
        spacing: 10px;          // 元素间距
        padding: 20px;          // 内边距
        
        Rectangle {
            background: red;
            min-width: 100px;   // 最小宽度
        }
        
        Rectangle {
            background: blue;
            min-width: 150px;
            // 默认会拉伸填充可用空间
        }
    }
}
```

#### 对齐方式

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    
    VerticalLayout {
        // stretch (默认): 拉伸填充
        HorizontalLayout {
            alignment: stretch;
            Rectangle { background: red; min-width: 50px; }
            Rectangle { background: blue; min-width: 50px; }
        }
        
        // start: 起始对齐
        HorizontalLayout {
            alignment: start;
            Rectangle { background: red; min-width: 50px; }
            Rectangle { background: blue; min-width: 50px; }
        }
        
        // end: 结束对齐
        HorizontalLayout {
            alignment: end;
            Rectangle { background: red; min-width: 50px; }
            Rectangle { background: blue; min-width: 50px; }
        }
        
        // center: 居中对齐
        HorizontalLayout {
            alignment: center;
            Rectangle { background: red; min-width: 50px; }
            Rectangle { background: blue; min-width: 50px; }
        }
        
        // space-between: 两端对齐,中间等距
        HorizontalLayout {
            alignment: space-between;
            Rectangle { background: red; min-width: 50px; }
            Rectangle { background: blue; min-width: 50px; }
        }
        
        // space-around: 周围等距
        HorizontalLayout {
            alignment: space-around;
            Rectangle { background: red; min-width: 50px; }
            Rectangle { background: blue; min-width: 50px; }
        }
    }
}
```

**类比说明**: 类似于 CSS Flexbox 的 `justify-content` 属性

#### 拉伸因子

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 100px;
    
    HorizontalLayout {
        // 默认拉伸因子为 1,等分空间
        Rectangle { background: red; }
        Rectangle { background: blue; }
        Rectangle { background: green; }
    }
}

export component CustomStretch inherits Window {
    preferred-width: 400px;
    preferred-height: 100px;
    
    HorizontalLayout {
        // 自定义拉伸因子
        Rectangle {
            background: red;
            horizontal-stretch: 2;  // 占用2份
        }
        Rectangle {
            background: blue;
            horizontal-stretch: 1;  // 占用1份
        }
        Rectangle {
            background: green;
            horizontal-stretch: 1;  // 占用1份
        }
        // 比例 red:blue:green = 2:1:1
    }
}
```

**拉伸算法**:
1. 先分配最小尺寸
2. 剩余空间按拉伸因子比例分配
3. 不超过最大尺寸限制

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 200px;
    
    VerticalLayout {
        // 等分空间
        HorizontalLayout {
            Rectangle { background: blue; }
            Rectangle { background: yellow; }
            Rectangle { background: green; }
        }
        
        // 有最小宽度的元素先满足最小宽度,再分配剩余空间
        HorizontalLayout {
            Rectangle { background: cyan; min-width: 100px; }
            Rectangle { background: magenta; min-width: 50px; }
            Rectangle { background: gold; }
        }
        
        // 拉伸因子为2的元素获得两倍空间
        HorizontalLayout {
            Rectangle { background: navy; horizontal-stretch: 2; }
            Rectangle { background: gray; }
        }
        
        // 有最大宽度限制的元素不会继续拉伸
        HorizontalLayout {
            Rectangle { background: red; max-width: 20px; }
            Rectangle { background: orange; horizontal-stretch: 0; }
            Rectangle { background: pink; horizontal-stretch: 0; }
        }
    }
}
```

### 5.4 网格布局

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    
    GridLayout {
        spacing: 5px;
        padding: 10px;
        
        // 方式1: 使用 Row 元素
        Row {
            Rectangle { background: red; }
            Rectangle { background: blue; }
        }
        Row {
            Rectangle { background: yellow; }
            Rectangle { background: green; }
        }
    }
}

export component GridWithRowCol inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    
    GridLayout {
        spacing: 5px;
        
        // 方式2: 使用 row 和 col 属性
        Rectangle { background: red; }      // row: 0, col: 0
        Rectangle { background: blue; }     // row: 0, col: 1
        Rectangle { background: yellow; row: 1; }  // row: 1, col: 0
        Rectangle { background: green; }    // row: 1, col: 1
        
        // 跨列显示
        Rectangle {
            background: black;
            col: 2;
            row: 0;
            rowspan: 2;  // 跨2行
        }
    }
}
```

**坑点警告**:
- `row` 和 `col` 必须是编译时常量,不能使用算术或依赖属性
- 网格布局目前不支持 `for` 或 `if` 表达式
- 不能混用 `Row` 元素和显式的 `row`/`col` 属性

### 5.5 相对长度(百分比)

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    
    background: green;
    
    // 完整写法
    Rectangle {
        background: blue;
        width: parent.width * 50%;
        height: parent.height * 50%;
    }
    
    // 简写形式(仅适用于 width 和 height)
    Rectangle {
        x: 200px;
        background: red;
        width: 50%;   // 等同于 parent.width * 50%
        height: 50%;  // 等同于 parent.height * 50%
    }
}
```

**简写条件**:
1. 属性必须是 `width` 或 `height`
2. 绑定表达式的结果是百分比

### 5.6 嵌套布局

```slint
export component Example inherits Window {
    preferred-width: 600px;
    preferred-height: 400px;
    
    HorizontalLayout {
        // 侧边栏
        Rectangle {
            background: #2c3e50;
            width: 200px;
        }
        
        // 主内容区
        VerticalLayout {
            padding: 0px;
            
            // 工具栏
            Rectangle {
                background: #34495e;
                height: 50px;
            }
            
            // 内容区域
            Rectangle {
                background: #ecf0f1;
                
                HorizontalLayout {
                    spacing: 10px;
                    padding: 20px;
                    
                    Rectangle {
                        background: white;
                        border-color: #bdc3c7;
                        border-width: 1px;
                    }
                    
                    Rectangle {
                        background: white;
                        border-color: #bdc3c7;
                        border-width: 1px;
                    }
                }
            }
            
            // 状态栏
            Rectangle {
                background: #2c3e50;
                height: 30px;
            }
        }
    }
}
```

### 5.7 布局中的条件和循环

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 100px;
    
    in-out property <bool> show-extra: false;
    
    HorizontalLayout {
        Rectangle { background: green; }
        
        // 条件元素
        if show-extra : Rectangle { background: yellow; }
        
        // 循环
        for color in [red, blue, purple] : Rectangle {
            background: color;
        }
        
        Rectangle { background: orange; }
    }
}
```

### 5.8 容器组件与 @children

```slint
// 创建可重用的容器组件
component BoxWithLabel inherits GridLayout {
    Row {
        Text {
            text: "Label:";
            color: #333;
        }
    }
    Row {
        @children  // 子元素将插入这里
    }
}

export component App inherits Window {
    preferred-width: 300px;
    preferred-height: 200px;
    
    VerticalLayout {
        BoxWithLabel {
            // 这些元素会被放置在 @children 的位置
            Rectangle {
                background: blue;
                height: 50px;
            }
        }
        
        BoxWithLabel {
            Text {
                text: "Custom content";
            }
        }
    }
}
```

**类比说明**: `@children` 类似于 React 的 `props.children` 或 Vue 的 `<slot>`

**最佳实践**:
- 使用 `@children` 创建可复用的布局容器
- 可以在 `@children` 周围添加装饰性元素(如边框、标签)

### 5.9 布局约束属性

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    
    HorizontalLayout {
        Rectangle {
            background: red;
            min-width: 100px;       // 最小宽度
            max-width: 200px;       // 最大宽度
            preferred-width: 150px; // 首选宽度
            horizontal-stretch: 1;  // 拉伸因子
        }
        
        Rectangle {
            background: blue;
            min-height: 50px;
            max-height: 150px;
            vertical-stretch: 2;
        }
    }
}
```

**约束优先级**:
1. `min-*` 优先级最高
2. `max-*` 次之
3. 拉伸分配在约束范围内进行

### 5.10 细粒度内边距控制

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    
    VerticalLayout {
        // 统一内边距
        padding: 20px;
        
        Rectangle { background: red; }
    }
    
    VerticalLayout {
        // 分别控制
        padding-left: 10px;
        padding-right: 10px;
        padding-top: 5px;
        padding-bottom: 5px;
        
        Rectangle { background: blue; }
    }
}
```

---

## 6. 全局单例

### 6.1 声明全局单例

全局单例用于在整个应用中共享状态和回调。

```slint
// 定义全局调色板
global Palette {
    in-out property <color> primary: #3498db;
    in-out property <color> secondary: #2ecc71;
    in-out property <color> accent: #e74c3c;
    in-out property <color> background: #ecf0f1;
    in-out property <color> text: #2c3e50;
}

// 在任何组件中使用
export component Button inherits Rectangle {
    background: Palette.primary;
    border-color: Palette.accent;
    border-width: 2px;
    
    Text {
        text: "Click Me";
        color: Palette.text;
    }
}

export component App inherits Window {
    background: Palette.background;
    
    Button { }
}
```

### 6.2 导出全局单例

```slint
// logic.slint
export global AppLogic {
    in-out property <int> user-score: 0;
    in-out property <string> username: "Guest";
    
    // 纯函数回调
    pure callback calculate-level(int) -> string;
    
    // 普通回调
    callback save-score();
}

// app.slint
import { AppLogic } from "logic.slint";

export { AppLogic }  // 重新导出,使其可从原生代码访问

export component App inherits Window {
    VerticalLayout {
        Text {
            text: "Score: " + AppLogic.user-score;
        }
        
        Text {
            text: "Level: " + AppLogic.calculate-level(AppLogic.user-score);
        }
    }
}
```

### 6.3 在原生代码中访问全局单例

**Rust 示例**:
```rust
slint::slint! {
    export global Logic {
        in-out property <int> the-value;
        pure callback magic-operation(int) -> int;
    }
    
    export component App inherits Window {
        // ...
    }
}

fn main() {
    let app = App::new().unwrap();
    
    // 设置回调
    app.global::<Logic>().on_magic_operation(|value| {
        println!("Input: {}", value);
        value * 2
    });
    
    // 设置属性
    app.global::<Logic>().set_the_value(42);
    
    // 读取属性
    let value = app.global::<Logic>().get_the_value();
    println!("Value: {}", value);
    
    app.run().unwrap();
}
```

**C++ 示例**:
```cpp
#include "app.h"

int main() {
    auto app = App::create();
    
    app->global<Logic>().on_magic_operation([](int value) -> int {
        return value * 2;
    });
    
    app->global<Logic>().set_the_value(42);
    
    app->run();
    return 0;
}
```

### 6.4 双向绑定全局属性

```slint
global Settings {
    in-out property <bool> dark-mode: false;
    in-out property <int> font-size: 14;
}

component SettingsPanel inherits Rectangle {
    // 在子组件中访问
    Text {
        text: Settings.dark-mode ? "Dark Mode ON" : "Dark Mode OFF";
    }
}

export component MainWindow inherits Window {
    // 重新暴露全局属性,方便原生代码访问
    in-out property dark-mode <=> Settings.dark-mode;
    in-out property font-size <=> Settings.font-size;
    
    SettingsPanel { }
}
```

**类比说明**: 全局单例类似于:
- React 的 Context API
- Vue 的 Vuex/Pinia 状态管理
- Angular 的 Service

**最佳实践**:
1. 用全局单例管理应用级状态(主题、用户信息等)
2. 用全局单例定义应用级回调(API 调用、数据持久化等)
3. 避免过度使用全局状态,保持组件的独立性
4. 导出需要从原生代码访问的全局单例

---

## 7. 重复与数据模型

### 7.1 基本 for 循环

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 100px;
    
    // 重复固定次数
    HorizontalLayout {
        for i in 5 : Rectangle {
            background: blue;
            width: 50px;
        }
    }
}
```

### 7.2 遍历数组

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 100px;
    
    // 遍历颜色数组
    HorizontalLayout {
        for color in [red, green, blue, yellow] : Rectangle {
            background: color;
            width: 80px;
        }
    }
}
```

### 7.3 带索引的循环

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 200px;
    
    VerticalLayout {
        for color[index] in [#e74c3c, #3498db, #2ecc71] : Rectangle {
            background: color;
            height: 60px;
            
            Text {
                text: "Item " + index;  // 索引从 0 开始
                color: white;
            }
        }
    }
}
```

### 7.4 遍历结构体数组

```slint
export component Example inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    
    // 定义数据模型
    in property <[{name: string, score: int, color: color}]> players: [
        { name: "Alice", score: 95, color: #e74c3c },
        { name: "Bob", score: 87, color: #3498db },
        { name: "Charlie", score: 92, color: #2ecc71 },
    ];
    
    VerticalLayout {
        spacing: 10px;
        padding: 20px;
        
        for player[index] in players : Rectangle {
            background: player.color;
            height: 60px;
            border-radius: 5px;
            
            HorizontalLayout {
                padding: 10px;
                
                Text {
                    text: (index + 1) + ". " + player.name;
                    color: white;
                    horizontal-stretch: 1;
                }
                
                Text {
                    text: "Score: " + player.score;
                    color: white;
                }
            }
        }
    }
}
```

### 7.5 数组属性和操作

```slint
export component Example inherits Window {
    preferred-width: 300px;
    preferred-height: 200px;
    
    // 声明数组属性
    in-out property <[int]> numbers: [1, 2, 3, 4, 5];
    in-out property <[{a: int, b: string}]> structs: [
        { a: 1, b: "hello" },
        { a: 2, b: "world" }
    ];
    
    VerticalLayout {
        Text {
            text: "Array length: " + numbers.length;
        }
        
        Text {
            text: "First element: " + numbers[0];
        }
        
        Text {
            text: "Third element: " + numbers[2];
        }
        
        // 遍历显示
        for num[idx] in numbers : Text {
            text: "numbers[" + idx + "] = " + num;
        }
    }
}
```

**数组操作**:
- `.length` - 获取数组长度
- `[index]` - 访问指定索引的元素
- 越界访问返回默认值(0, "", false 等)

**坑点警告**:
```slint
// ❌ 数组索引越界不会报错,返回默认值
property <[int]> nums: [1, 2, 3];
property <int> invalid: nums[10];  // 返回 0,不报错!

// ❌ 不能直接修改数组元素
// nums[0] = 5;  // 错误!

// ✅ 需要重新赋值整个数组
callback update-array;
update-array => {
    nums = [5, 2, 3];  // 正确
}
```

### 7.6 动态数据模型

从原生代码更新数组:

```slint
export component TodoApp inherits Window {
    preferred-width: 400px;
    preferred-height: 500px;
    
    in-out property <[{text: string, completed: bool}]> todos;
    
    callback add-todo(string);
    callback toggle-todo(int);
    
    VerticalLayout {
        padding: 20px;
        spacing: 10px;
        
        Text {
            text: "Todo List (" + todos.length + " items)";
            font-size: 20px;
        }
        
        for todo[index] in todos : Rectangle {
            height: 40px;
            background: todo.completed ? #2ecc71 : #ecf0f1;
            border-radius: 5px;
            
            HorizontalLayout {
                padding: 10px;
                
                Text {
                    text: todo.text;
                    color: todo.completed ? white : black;
                    horizontal-stretch: 1;
                }
            }
            
            TouchArea {
                clicked => {
                    root.toggle-todo(index);
                }
            }
        }
    }
}
```

**Rust 原生代码**:
```rust
let app = TodoApp::new().unwrap();

// 初始化数据
let todos = std::rc::Rc::new(slint::VecModel::from(vec![
    TodoItem { text: "Learn Slint".into(), completed: false },
    TodoItem { text: "Build UI".into(), completed: false },
]));
app.set_todos(todos.clone().into());

// 添加项目
app.on_add_todo({
    let todos = todos.clone();
    move |text| {
        todos.push(TodoItem {
            text: text.to_string(),
            completed: false,
        });
    }
});

// 切换完成状态
app.on_toggle_todo({
    let todos = todos.clone();
    move |index| {
        let mut item = todos.row_data(index as usize).unwrap();
        item.completed = !item.completed;
        todos.set_row_data(index as usize, item);
    }
});
```

**类比说明**: Slint 的数组类似于:
- React 的 state 数组
- Vue 的响应式数组
- 但更新需要通过原生代码的 Model API

---

## 8. 动画系统

### 8.1 基本动画

```slint
export component Example inherits Window {
    preferred-width: 200px;
    preferred-height: 200px;
    
    background: area.pressed ? #3498db : #e74c3c;
    
    // 动画背景颜色变化,持续 250 毫秒
    animate background {
        duration: 250ms;
    }
    
    area := TouchArea { }
}
```

### 8.2 动画多个属性

```slint
export component Example inherits Window {
    preferred-width: 300px;
    preferred-height: 300px;
    
    rect := Rectangle {
        x: area.pressed ? 200px : 50px;
        y: area.pressed ? 200px : 50px;
        width: 50px;
        height: 50px;
        background: area.pressed ? blue : red;
        
        // 方式1: 分别动画
        animate x {
            duration: 500ms;
            easing: ease-in-out;
        }
        animate y {
            duration: 500ms;
            easing: ease-in-out;
        }
        
        // 方式2: 同时动画(推荐)
        animate x, y {
            duration: 500ms;
            easing: ease-in-out;
        }
        
        // 背景色单独动画
        animate background {
            duration: 300ms;
        }
    }
    
    area := TouchArea { }
}
```

### 8.3 动画参数详解

```slint
export component Example inherits Window {
    preferred-width: 200px;
    preferred-height: 200px;
    
    rect := Rectangle {
        x: area.pressed ? 150px : 50px;
        background: red;
        width: 50px;
        height: 50px;
        
        animate x {
            // 延迟启动(默认: 0ms)
            delay: 100ms;
            
            // 动画持续时间(默认: 0ms)
            duration: 500ms;
            
            // 重复次数(默认: 1.0)
            // 负数表示无限重复
            // 可以是小数,如 2.5 表示重复2.5次
            iteration-count: 1;
            
            // 缓动函数(默认: linear)
            easing: ease-in-out;
            
            // 动画方向(默认: normal)
            direction: normal;
        }
    }
    
    area := TouchArea { }
}
```

### 8.4 缓动函数 (Easing)

Slint 支持多种缓动函数(参考 [easings.net](https://easings.net/)):

```slint
export component EasingDemo inherits Window {
    preferred-width: 800px;
    preferred-height: 600px;
    
    in-out property <bool> animate-trigger: false;
    
    VerticalLayout {
        padding: 20px;
        spacing: 15px;
        
        // Linear: 线性,匀速
        Rectangle {
            height: 30px;
            background: #e74c3c;
            width: animate-trigger ? 700px : 50px;
            animate width { duration: 1s; easing: linear; }
            Text { text: "linear"; color: white; }
        }
        
        // Ease: 慢-快-慢
        Rectangle {
            height: 30px;
            background: #3498db;
            width: animate-trigger ? 700px : 50px;
            animate width { duration: 1s; easing: ease; }
            Text { text: "ease"; color: white; }
        }
        
        // Ease-in: 慢速开始
        Rectangle {
            height: 30px;
            background: #2ecc71;
            width: animate-trigger ? 700px : 50px;
            animate width { duration: 1s; easing: ease-in; }
            Text { text: "ease-in"; color: white; }
        }
        
        // Ease-out: 慢速结束
        Rectangle {
            height: 30px;
            background: #f39c12;
            width: animate-trigger ? 700px : 50px;
            animate width { duration: 1s; easing: ease-out; }
            Text { text: "ease-out"; color: white; }
        }
        
        // Ease-in-out: 慢速开始和结束
        Rectangle {
            height: 30px;
            background: #9b59b6;
            width: animate-trigger ? 700px : 50px;
            animate width { duration: 1s; easing: ease-in-out; }
            Text { text: "ease-in-out"; color: white; }
        }
        
        // Cubic-bezier: 自定义贝塞尔曲线
        Rectangle {
            height: 30px;
            background: #1abc9c;
            width: animate-trigger ? 700px : 50px;
            animate width {
                duration: 1s;
                easing: cubic-bezier(0.68, -0.55, 0.265, 1.55);
            }
            Text { text: "cubic-bezier (bounce)"; color: white; }
        }
        
        // Ease-out-bounce: 弹跳效果
        Rectangle {
            height: 30px;
            background: #34495e;
            width: animate-trigger ? 700px : 50px;
            animate width { duration: 1s; easing: ease-out-bounce; }
            Text { text: "ease-out-bounce"; color: white; }
        }
    }
    
    TouchArea {
        clicked => {
            animate-trigger = !animate-trigger;
        }
    }
}
```

**常用缓动函数**:
- `linear` - 线性
- `ease` - 默认缓动
- `ease-in` - 加速
- `ease-out` - 减速
- `ease-in-out` - 加速后减速
- `ease-in-quad`, `ease-in-cubic`, `ease-in-quart` - 不同程度的加速
- `ease-out-bounce` - 弹跳效果
- `ease-out-elastic` - 弹性效果
- `cubic-bezier(x1, y1, x2, y2)` - 自定义贝塞尔曲线

### 8.5 动画方向

```slint
export component DirectionDemo inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    
    in-out property <bool> running: true;
    
    VerticalLayout {
        padding: 20px;
        spacing: 20px;
        
        // normal: 正常方向
        Rectangle {
            height: 50px;
            background: red;
            x: 0;
            animate x {
                duration: 2s;
                iteration-count: -1;  // 无限循环
                direction: normal;
            }
            init => {
                if running {
                    self.x = 300px;
                }
            }
        }
        
        // reverse: 反向
        Rectangle {
            height: 50px;
            background: blue;
            x: 0;
            animate x {
                duration: 2s;
                iteration-count: -1;
                direction: reverse;
            }
            init => {
                if running {
                    self.x = 300px;
                }
            }
        }
        
        // alternate: 交替方向
        Rectangle {
            height: 50px;
            background: green;
            x: 0;
            animate x {
                duration: 2s;
                iteration-count: -1;
                direction: alternate;
            }
            init => {
                if running {
                    self.x = 300px;
                }
            }
        }
        
        // alternate-reverse: 反向交替
        Rectangle {
            height: 50px;
            background: orange;
            x: 0;
            animate x {
                duration: 2s;
                iteration-count: -1;
                direction: alternate-reverse;
            }
            init => {
                if running {
                    self.x = 300px;
                }
            }
        }
    }
}
```

**动画方向说明**:
- `normal` - 从起点到终点
- `reverse` - 从终点到起点
- `alternate` - 奇数次正向,偶数次反向
- `alternate-reverse` - 奇数次反向,偶数次正向

### 8.6 无限动画与 animation-tick()

对于需要持续运行的动画,使用 `animation-tick()` 函数:

```slint
export component LoadingSpinner inherits Window {
    preferred-width: 200px;
    preferred-height: 200px;
    
    Rectangle {
        width: 100px;
        height: 100px;
        
        // 使用 animation-tick() 创建持续旋转
        rotation-angle: animation-tick() * 360deg / 1s;
        
        Rectangle {
            y: 0;
            x: parent.width / 2 - self.width / 2;
            width: 10px;
            height: 30px;
            background: #3498db;
            border-radius: 5px;
        }
    }
}
```

**类比说明**: `animation-tick()` 类似于 CSS 的 `@keyframes` 或 JavaScript 的 `requestAnimationFrame`

**最佳实践**:
1. 简单的状态变化用 `animate`
2. 复杂的序列动画考虑状态机(见下一章)
3. 持续运行的动画用 `animation-tick()`
4. 避免同时动画过多属性,影响性能

---

## 9. 状态管理

### 9.1 基本状态定义

```slint
export component Example inherits Window {
    preferred-width: 200px;
    preferred-height: 200px;
    default-font-size: 20px;
    
    label := Text { }
    
    ta := TouchArea {
        clicked => {
            active = !active;
        }
    }
    
    property <bool> active: true;
    
    // 定义状态
    states [
        active when active && !ta.has-hover: {
            label.text: "Active";
            root.background: #3498db;
        }
        active-hover when active && ta.has-hover: {
            label.text: "Active\nHover";
            root.background: #2ecc71;
        }
        inactive when !active: {
            label.text: "Inactive";
            root.background: #95a5a6;
        }
    ]
}
```

**状态语法**:
```
state-name when condition : {
    element.property: value;
}
```

### 9.2 状态转换与动画

```slint
export component Button inherits Rectangle {
    preferred-width: 150px;
    preferred-height: 50px;
    
    in property <string> text: "Click Me";
    in-out property <bool> pressed;
    in-out property <bool> enabled: true;
    
    label := Text {
        text: root.text;
    }
    
    touch := TouchArea {
        clicked => {
            if root.enabled {
                pressed = !pressed;
            }
        }
    }
    
    states [
        disabled when !enabled : {
            background: #bdc3c7;
            label.color: #7f8c8d;
            
            // out: 离开此状态时的动画
            out {
                animate * {
                    duration: 800ms;
                    easing: ease-out;
                }
            }
        }
        
        normal when enabled && !pressed && !touch.has-hover: {
            background: #3498db;
            label.color: white;
        }
        
        hover when enabled && !pressed && touch.has-hover: {
            background: #2980b9;
            label.color: white;
            
            // in: 进入此状态时的动画
            in {
                animate background {
                    duration: 200ms;
                    easing: ease-in;
                }
            }
        }
        
        pressed when enabled && pressed: {
            background: #1abc9c;
            label.color: white;
            label.text: "Pressed!";
            
            // in: 进入此状态时的动画
            in {
                animate background {
                    duration: 150ms;
                }
            }
            
            // out: 离开此状态时的动画
            out {
                animate label.text {
                    duration: 300ms;
                }
            }
        }
    ]
}

export component App inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    
    VerticalLayout {
        padding: 20px;
        spacing: 20px;
        
        Button {
            text: "Normal Button";
        }
        
        Button {
            text: "Disabled Button";
            enabled: false;
        }
    }
}
```

### 9.3 转换类型详解

```slint
export component StateTransitionDemo inherits Rectangle {
    preferred-width: 300px;
    preferred-height: 400px;
    
    in-out property <int> mode: 0;
    
    callback next-mode;
    next-mode => {
        mode = (mode + 1) % 3;
    }
    
    box := Rectangle {
        width: 100px;
        height: 100px;
        x: 100px;
        y: 50px;
    }
    
    states [
        red-state when mode == 0: {
            box.background: #e74c3c;
            box.x: 20px;
            box.y: 50px;
            
            // in: 只在进入时动画
            in {
                animate box.background, box.x, box.y {
                    duration: 500ms;
                    easing: ease-in;
                }
            }
        }
        
        blue-state when mode == 1: {
            box.background: #3498db;
            box.x: 180px;
            box.y: 150px;
            
            // out: 只在离开时动画
            out {
                animate box.background, box.x, box.y {
                    duration: 500ms;
                    easing: ease-out;
                }
            }
        }
        
        green-state when mode == 2: {
            box.background: #2ecc71;
            box.x: 100px;
            box.y: 250px;
            
            // in-out: 进入和离开时都动画
            in-out {
                animate box.background, box.x, box.y {
                    duration: 500ms;
                    easing: ease-in-out;
                }
            }
        }
    ]
    
    VerticalLayout {
        y: 300px;
        Button {
            text: "Next State";
            clicked => {
                root.next-mode();
            }
        }
    }
}
```

**转换类型对比**:
- `in`: 进入状态时动画 → 适合强调状态开始
- `out`: 离开状态时动画 → 适合强调状态结束
- `in-out`: 双向动画 → 适合对称的状态切换

### 9.4 复杂状态机示例

```slint
export component TrafficLight inherits Window {
    preferred-width: 200px;
    preferred-height: 400px;
    background: #2c3e50;
    
    // 0: red, 1: yellow, 2: green
    in-out property <int> state: 0;
    
    callback advance;
    advance => {
        state = (state + 1) % 3;
    }
    
    VerticalLayout {
        padding: 40px;
        spacing: 30px;
        alignment: center;
        
        // 红灯
        red-light := Rectangle {
            width: 100px;
            height: 100px;
            border-radius: 50px;
            border-width: 3px;
            border-color: #34495e;
        }
        
        // 黄灯
        yellow-light := Rectangle {
            width: 100px;
            height: 100px;
            border-radius: 50px;
            border-width: 3px;
            border-color: #34495e;
        }
        
        // 绿灯
        green-light := Rectangle {
            width: 100px;
            height: 100px;
            border-radius: 50px;
            border-width: 3px;
            border-color: #34495e;
        }
    }
    
    states [
        red when state == 0: {
            red-light.background: #e74c3c;
            yellow-light.background: #34495e;
            green-light.background: #34495e;
            
            in {
                animate red-light.background {
                    duration: 300ms;
                }
            }
        }
        
        yellow when state == 1: {
            red-light.background: #34495e;
            yellow-light.background: #f39c12;
            green-light.background: #34495e;
            
            in {
                animate yellow-light.background {
                    duration: 300ms;
                }
            }
        }
        
        green when state == 2: {
            red-light.background: #34495e;
            yellow-light.background: #34495e;
            green-light.background: #2ecc71;
            
            in {
                animate green-light.background {
                    duration: 300ms;
                }
            }
        }
    ]
    
    TouchArea {
        clicked => {
            root.advance();
        }
    }
}
```

### 9.5 状态驱动的表单验证

```slint
export component LoginForm inherits Window {
    preferred-width: 400px;
    preferred-height: 500px;
    
    in-out property <string> username;
    in-out property <string> password;
    in-out property <bool> is-loading: false;
    in-out property <bool> has-error: false;
    in-out property <string> error-message;
    
    // 表单状态判断
    property <bool> is-valid: username.length >= 3 && password.length >= 6;
    property <bool> is-idle: !is-loading && !has-error;
    property <bool> show-success: !is-loading && !has-error && username.length > 0;
    
    callback submit;
    
    VerticalLayout {
        padding: 40px;
        spacing: 20px;
        
        Text {
            text: "Login";
            font-size: 24px;
            horizontal-alignment: center;
        }
        
        // 用户名输入框
        username-input := Rectangle {
            height: 50px;
            border-radius: 5px;
            
            LineEdit {
                text <=> username;
                placeholder-text: "Username";
            }
        }
        
        // 密码输入框
        password-input := Rectangle {
            height: 50px;
            border-radius: 5px;
            
            LineEdit {
                text <=> password;
                placeholder-text: "Password";
                input-type: password;
            }
        }
        
        // 提交按钮
        submit-btn := Rectangle {
            height: 50px;
            border-radius: 5px;
            
            Text {
                text: "Login";
                color: white;
                horizontal-alignment: center;
                vertical-alignment: center;
            }
            
            TouchArea {
                clicked => {
                    if is-valid {
                        root.submit();
                    }
                }
            }
        }
        
        // 错误消息
        if has-error : error-text := Text {
            text: error-message;
            color: #e74c3c;
            horizontal-alignment: center;
        }
    }
    
    states [
        idle when is-idle: {
            username-input.background: #ecf0f1;
            password-input.background: #ecf0f1;
            submit-btn.background: is-valid ? #3498db : #bdc3c7;
        }
        
        loading when is-loading: {
            username-input.background: #ecf0f1;
            password-input.background: #ecf0f1;
            submit-btn.background: #95a5a6;
            
            in {
                animate submit-btn.background {
                    duration: 200ms;
                }
            }
        }
        
        error when has-error: {
            username-input.background: #fadbd8;
            password-input.background: #fadbd8;
            submit-btn.background: #e74c3c;
            
            in {
                animate username-input.background, password-input.background {
                    duration: 300ms;
                }
            }
        }
    ]
}
```

**最佳实践**:
1. 使用状态管理复杂的 UI 状态(禁用、加载、错误等)
2. 通过条件表达式驱动状态切换
3. 使用转换动画使状态变化更流畅
4. 避免状态条件重叠(每次只应有一个状态激活)

**坑点警告**:
```slint
// ❌ 错误: 状态条件可能重叠
states [
    state-a when value > 0: { }
    state-b when value > 5: { }  // 当 value > 5 时两个条件都满足!
]

// ✅ 正确: 确保条件互斥
states [
    state-a when value > 0 && value <= 5: { }
    state-b when value > 5: { }
]
```

---

## 10. 函数与回调

### 10.1 声明函数

```slint
export component Calculator inherits Rectangle {
    // 基本函数
    function add(a: int, b: int) -> int {
        return a + b;
    }
    
    // 纯函数(无副作用)
    pure function multiply(x: int, y: int) -> int {
        return x * y;
    }
    
    // 无返回值的函数
    function log-message(msg: string) {
        debug("Message:", msg);
    }
    
    // 最后一个表达式自动返回
    pure function square(n: int) -> int {
        n * n  // 自动返回,不需要 return
    }
    
    Text {
        text: "2 + 3 = " + root.add(2, 3);
    }
}
```

**函数特性**:
- 使用 `function` 关键字声明
- 参数格式: `name: type`
- 返回类型: `-> type`
- 可标记为 `pure`(纯函数)

### 10.2 纯函数 (Pure Functions)

```slint
export component Example {
    // ✅ 纯函数: 只依赖参数,无副作用
    pure function calculate(x: int, y: int) -> int {
        return x * 2 + y;
    }
    
    // ❌ 非纯函数: 访问了属性
    property <int> multiplier: 2;
    function impure-calculate(x: int) -> int {
        return x * multiplier;  // 依赖外部属性
    }
    
    // ❌ 非纯函数: 有副作用
    property <int> counter: 0;
    function increment() {
        counter += 1;  // 修改状态
    }
}
```

**纯函数的好处**:
1. 可以被优化器更好地优化
2. 更容易测试和理解
3. 可以安全地并行执行
4. 结果可以被缓存

**最佳实践**: 尽可能将函数标记为 `pure`

### 10.3 调用函数

```slint
component MathUtils {
    public pure function double(x: int) -> int {
        return x * 2;
    }
    
    public pure function triple(x: int) -> int {
        return x * 3;
    }
}

export component Example inherits Rectangle {
    // 本地函数
    pure function local-func() -> int {
        return 42;
    }
    
    // 调用本地函数(无需元素名)
    property <int> value1: local-func();
    
    // 调用子元素的函数(需要元素名)
    math := MathUtils { }
    property <int> value2: math.double(21);
    
    Text {
        // 调用预定义元素
        text: "Result: " + root.local-func();
        
        // 在回调中调用
        TouchArea {
            clicked => {
                let result = root.math.triple(10);
                debug("Triple:", result);
            }
        }
    }
}
```

### 10.4 函数可见性

```slint
// math.slint
export component MathLib {
    // 私有函数(默认)
    function private-helper(x: int) -> int {
        return x * 2;
    }
    
    // 公有函数(可被其他组件访问)
    public pure function double(x: int) -> int {
        return private-helper(x);
    }
    
    // 受保护函数(只能被继承的组件访问)
    protected pure function internal-calc(x: int) -> int {
        return x * 3;
    }
}

// app.slint
import { MathLib } from "math.slint";

export component App inherits Window {
    math := MathLib { }
    
    property <int> result: math.double(5);  // ✅ OK: public
    // property <int> bad: math.private-helper(5);  // ❌ 错误: private
}
```

**可见性级别**:
- (默认): 私有,只能在组件内部访问
- `public`: 公有,可被任何组件访问,也可从原生代码调用
- `protected`: 受保护,只能被继承的组件访问

### 10.5 从原生代码调用函数

```slint
// ui/app.slint
export component App inherits Window {
    public pure function format-greeting(name: string) -> string {
        return "Hello, " + name + "!";
    }
    
    callback show-message(string);
}
```

**Rust 代码**:
```rust
fn main() {
    let app = App::new().unwrap();
    
    // 调用 Slint 函数
    let greeting = app.invoke_format_greeting("Alice".into());
    println!("{}", greeting);  // "Hello, Alice!"
    
    app.run().unwrap();
}
```

### 10.6 回调 (Callbacks)

```slint
export component Button inherits Rectangle {
    // 声明回调
    callback clicked;
    
    // 带参数的回调
    callback value-changed(int);
    
    // 带返回值的回调
    callback validate(string) -> bool;
    
    // 带命名参数的回调(提高可读性)
    callback user-action(username: string, action: string);
    
    TouchArea {
        clicked => {
            // 触发回调
            root.clicked();
            root.value-changed(42);
            
            // 调用带返回值的回调
            let is-valid = root.validate("test");
            if is-valid {
                root.user-action("Alice", "login");
            }
        }
    }
}

export component App inherits Window {
    btn := Button {
        // 设置回调处理器
        clicked => {
            debug("Button clicked!");
        }
        
        value-changed(val) => {
            debug("Value:", val);
        }
        
        validate(text) => {
            return text.length > 0;
        }
        
        user-action(user, act) => {
            debug("User:", user, "Action:", act);
        }
    }
}
```

### 10.7 回调别名

```slint
component InnerButton inherits Rectangle {
    callback clicked;
    
    TouchArea {
        clicked => {
            root.clicked();
        }
    }
}

export component OuterButton inherits Rectangle {
    // 使用 <=> 创建回调别名
    callback clicked <=> inner.clicked;
    
    inner := InnerButton { }
}

export component App inherits Window {
    outer := OuterButton {
        // 直接处理 outer 的 clicked,实际会触发 inner 的 clicked
        clicked => {
            debug("Clicked!");
        }
    }
}
```

**类比说明**: 回调别名类似于:
- React 的 prop drilling 解决方案
- Vue 的 v-model 双向绑定
- 事件冒泡机制

### 10.8 函数 vs 回调对比

```slint
export component Comparison inherits Rectangle {
    // 函数: 在 Slint 中定义,不能从外部设置
    pure function calculate(x: int) -> int {
        return x * 2;
    }
    
    // 回调: 可以从外部设置处理器
    callback on-calculate(int) -> int;
    
    property <int> result1: calculate(5);  // 直接调用函数
    
    callback trigger;
    trigger => {
        // 调用回调(如果外部设置了处理器)
        let result2 = on-calculate(5);
        debug("Callback result:", result2);
    }
}

export component App inherits Window {
    comp := Comparison {
        // 不能修改函数的实现
        // calculate(x) => { ... }  // ❌ 错误!
        
        // 可以设置回调处理器
        on-calculate(x) => {
            return x * 3;  // 自定义实现
        }
    }
}
```

**何时使用函数 vs 回调**:

| 特性 | 函数 | 回调 |
|------|------|------|
| 实现位置 | 必须在 Slint 中 | 可在原生代码中 |
| 可重定义 | 否 | 是 |
| 从原生代码调用 | 是(如果是 public) | 否 |
| 适用场景 | 内部逻辑、工具函数 | 事件处理、与原生交互 |

**最佳实践**:
- 纯计算逻辑 → 使用函数
- 需要外部处理 → 使用回调
- 需要从原生代码调用 → 使用 public 函数
- 事件通知 → 使用回调

---

## 11. 名称解析与作用域

### 11.1 名称查找规则

```slint
export component Example inherits Rectangle {
    // 根级函数
    public pure function root-func() -> int {
        return 1;
    }
    
    property <int> test1: root-func();        // 调用根级函数,结果: 1
    property <int> test2: self.root-func();   // 也是根级函数,结果: 1
    
    VerticalLayout {
        // 布局级函数
        public pure function layout-func() -> int {
            return 2;
        }
        
        Text {
            // Text 级函数
            public pure function text-func() -> int {
                return 3;
            }
            
            // 无元素名调用: 先找当前元素,再向上查找父元素
            text: "Current: " + text-func();     // 3 (当前元素)
            
            // 调用父元素的函数
            property <int> parent-val: layout-func();  // 需要在作用域内
        }
        
        Text {
            // 在这个 Text 中调用各级函数
            text: "Layout: " + layout-func() +        // 2 (父元素)
                  ", Root: " + root-func();           // 1 (向上查找到根)
            
            // text-func() 在这里不可用,因为它属于另一个 Text
        }
    }
}
```

**查找规则**:
1. 不带元素名调用 → 从当前元素向上查找
2. 带元素名调用(`element.function()`) → 只在指定元素查找
3. `self.function()` ≠ `function()` (self 不会向上查找)

### 11.2 函数遮蔽 (Shadowing)

```slint
export component Example inherits Rectangle {
    property <int> secret-number: my-function();
    
    // 根级函数
    public pure function my-function() -> int {
        return 1;
    }
    
    VerticalLayout {
        // 遮蔽根级函数
        public pure function my-function() -> int {
            return 2;
        }
        
        Text {
            // 再次遮蔽
            public pure function my-function() -> int {
                return 3;
            }
            
            text: "The secret number is " + my-function();  // 结果: 3
        }
        
        Text {
            text: "The other secret number is " + my-function();  // 结果: 2
        }
    }
    
    Text {
        text: "Root secret: " + my-function();  // 结果: 1
    }
}
```

**输出结果**:
- `secret-number` = 1
- 第一个 Text: "The secret number is 3"
- 第二个 Text: "The other secret number is 2"
- 第三个 Text: "Root secret: 1"

### 11.3 显式作用域访问

```slint
export component ScopeDemo inherits Window {
    property <string> level: "root";
    
    pure function get-info() -> string {
        return "Root level";
    }
    
    VerticalLayout {
        property <string> level: "layout";
        
        pure function get-info() -> string {
            return "Layout level";
        }
        
        Text {
            property <string> level: "text";
            
            pure function get-info() -> string {
                return "Text level";
            }
            
            // 不同的访问方式
            text: 
                "self.level: " + self.level +           // "text"
                "\nroot.level: " + root.level +         // "root"
                "\nparent.level: " + parent.level +     // "layout"
                "\nget-info(): " + get-info() +         // "Text level"
                "\nroot.get-info(): " + root.get-info();  // "Root level"
        }
    }
}
```

### 11.4 属性与函数的作用域差异

```slint
export component Example {
    property <int> my-property: 10;
    
    pure function my-function() -> int {
        // ✅ 函数可以访问属性
        return my-property * 2;
    }
    
    property <int> calculated: {
        // ✅ 属性绑定可以调用函数
        my-function() + 5
    }
    
    Rectangle {
        // ✅ 子元素可以访问父元素的属性
        width: root.my-property * 1px;
        
        // ✅ 子元素可以调用父元素的函数
        height: root.my-function() * 1px;
    }
}
```

### 11.5 局部变量作用域

```slint
export component Example {
    callback demo;
    
    demo => {
        let x = 10;
        
        if (true) {
            let y = 20;
            debug(x);  // ✅ OK: x 在外部作用域
            debug(y);  // ✅ OK: y 在当前作用域
        }
        
        // debug(y);  // ❌ 错误: y 超出作用域
        
        // let x = 5;  // ❌ 错误: 不能重新声明 x
    }
}
```

**坑点警告**:
```slint
// ❌ 即使在不同的 if 分支,也不能重新声明
callback bad-example;
bad-example => {
    if condition1 {
        let x = 1;
    }
    if condition2 {
        let x = 2;  // ❌ 错误!即使分支不重叠也不行
    }
}

// ✅ 正确: 使用不同的变量名
callback good-example;
good-example => {
    if condition1 {
        let x1 = 1;
    }
    if condition2 {
        let x2 = 2;  // ✅ OK
    }
}
```

### 11.6 最佳实践示例

```slint
// 工具函数组件
component MathUtils {
    // 将工具函数标记为 public
    public pure function clamp(value: int, min: int, max: int) -> int {
        if value < min {
            return min;
        }
        if value > max {
            return max;
        }
        return value;
    }
    
    public pure function lerp(a: float, b: float, t: float) -> float {
        return a + (b - a) * t;
    }
}

export component App inherits Window {
    // 实例化工具组件
    utils := MathUtils { }
    
    // 使用工具函数
    property <int> clamped-value: utils.clamp(150, 0, 100);  // 100
    
    Rectangle {
        // 在子元素中也可以访问
        width: root.utils.clamp(200, 50, 150) * 1px;  // 150px
    }
}
```

**命名最佳实践**:
1. 使用描述性的函数名
2. 公共函数用 `public` 明确标记
3. 工具函数集中在专门的组件中
4. 避免过度使用函数遮蔽
5. 使用 `root`, `self`, `parent` 明确作用域

---

## 12. 结构体与枚举

### 12.1 定义结构体

```slint
// 命名结构体
export struct Player {
    name: string,
    score: int,
    level: int,
    is-active: bool,
}

export component Example {
    in-out property <Player> current-player: {
        name: "Alice",
        score: 1500,
        level: 5,
        is-active: true,
    };
    
    Text {
        text: current-player.name + " - Score: " + current-player.score;
    }
}
```

**结构体特性**:
- 使用 `struct` 关键字定义
- 字段用逗号分隔
- 最后一个字段后可以有逗号(推荐)
- 默认值: 所有字段都是其类型的默认值

### 12.2 匿名结构体

```slint
export component Example {
    // 内联定义结构体类型
    in-out property <{name: string, age: int}> person: {
        name: "Bob",
        age: 30,
    };
    
    // 嵌套结构体
    in-out property <{
        user: {name: string, email: string},
        settings: {theme: string, notifications: bool}
    }> app-data: {
        user: {name: "Charlie", email: "charlie@example.com"},
        settings: {theme: "dark", notifications: true},
    };
    
    Text {
        text: app-data.user.name + " - " + app-data.settings.theme;
    }
}
```

### 12.3 结构体数组

```slint
export struct Task {
    title: string,
    completed: bool,
    priority: int,
}

export component TodoList inherits Window {
    preferred-width: 400px;
    preferred-height: 500px;
    
    in-out property <[Task]> tasks: [
        {title: "学习 Slint", completed: false, priority: 1},
        {title: "写文档", completed: false, priority: 2},
        {title: "测试应用", completed: true, priority: 3},
    ];
    
    VerticalLayout {
        padding: 20px;
        spacing: 10px;
        
        for task[index] in tasks : Rectangle {
            height: 60px;
            background: task.completed ? #2ecc71 : #ecf0f1;
            border-radius: 5px;
            
            HorizontalLayout {
                padding: 15px;
                spacing: 10px;
                
                Text {
                    text: "📌";
                    font-size: 20px;
                }
                
                VerticalLayout {
                    spacing: 5px;
                    Text {
                        text: task.title;
                        font-weight: 700;
                    }
                    Text {
                        text: "Priority: " + task.priority;
                        font-size: 12px;
                        color: #7f8c8d;
                    }
                }
            }
        }
    }
}
```

### 12.4 定义枚举

```slint
// 简单枚举
export enum CardSuit {
    clubs,
    diamonds,
    hearts,
    spade,
}

// 状态枚举
export enum LoadingState {
    idle,
    loading,
    success,
    error,
}

export component Example {
    in-out property <CardSuit> current-suit: CardSuit.spade;
    in-out property <LoadingState> state: LoadingState.idle;
    
    Text {
        text: "Current suit: " + current-suit;  // 显示枚举值
    }
    
    // 枚举比较
    property <bool> is-hearts: current-suit == CardSuit.hearts;
}
```

### 12.5 枚举简写

在特定上下文中,可以省略枚举名称:

```slint
export enum Theme {
    light,
    dark,
    auto,
}

export component Example {
    in-out property <Theme> current-theme: Theme.dark;
    
    callback set-theme(Theme);
    
    set-theme(theme) => {
        // 在赋值时可以省略枚举名
        current-theme = theme;
    }
    
    // 在同类型绑定中可以省略
    property <Theme> default-theme: dark;  // 等同于 Theme.dark
    
    // 在比较时必须使用完整名称
    property <bool> is-dark: current-theme == Theme.dark;
}
```

**简写规则**:
1. 赋值给枚举类型属性时可以省略
2. 作为枚举类型回调参数时可以省略
3. 比较时需要完整的 `EnumName.value`
4. 返回枚举值的回调中可以省略

### 12.6 枚举默认值

```slint
export enum Status {
    pending,      // 这是默认值(第一个)
    approved,
    rejected,
}

export component Example {
    // 未指定值时,使用第一个枚举值作为默认值
    property <Status> current-status;  // 默认为 Status.pending
    
    Text {
        text: {
            if current-status == Status.pending {
                "⏳ Pending"
            } else if current-status == Status.approved {
                "✅ Approved"
            } else {
                "❌ Rejected"
            }
        };
    }
}
```

**重要**: 枚举的默认值**总是第一个定义的值**

### 12.7 实际应用示例 - 状态机

```slint
export enum AppState {
    login,
    dashboard,
    settings,
    logout,
}

export struct User {
    username: string,
    email: string,
    avatar-url: string,
}

export component App inherits Window {
    preferred-width: 800px;
    preferred-height: 600px;
    
    in-out property <AppState> current-state: AppState.login;
    in-out property <User> current-user;
    
    callback navigate(AppState);
    callback login(string, string);
    
    navigate(new-state) => {
        current-state = new-state;
    }
    
    // 根据状态显示不同界面
    if current-state == AppState.login : LoginScreen {
        width: 100%;
        height: 100%;
    }
    
    if current-state == AppState.dashboard : DashboardScreen {
        width: 100%;
        height: 100%;
        user: current-user;
    }
    
    if current-state == AppState.settings : SettingsScreen {
        width: 100%;
        height: 100%;
    }
}

component LoginScreen inherits Rectangle {
    background: #ecf0f1;
    
    VerticalLayout {
        alignment: center;
        
        Text {
            text: "Login";
            font-size: 32px;
            horizontal-alignment: center;
        }
    }
}

component DashboardScreen inherits Rectangle {
    in property <User> user;
    background: white;
    
    VerticalLayout {
        padding: 20px;
        
        Text {
            text: "Welcome, " + user.username;
            font-size: 24px;
        }
    }
}

component SettingsScreen inherits Rectangle {
    background: #f8f9fa;
}
```

### 12.8 结构体与枚举的组合使用

```slint
export enum MessageType {
    info,
    warning,
    error,
    success,
}

export struct Message {
    type: MessageType,
    title: string,
    content: string,
    timestamp: int,
}

export component NotificationCenter inherits Window {
    preferred-width: 400px;
    preferred-height: 600px;
    
    in-out property <[Message]> messages: [
        {
            type: MessageType.info,
            title: "系统通知",
            content: "您有新的更新可用",
            timestamp: 1234567890,
        },
        {
            type: MessageType.warning,
            title: "警告",
            content: "磁盘空间不足",
            timestamp: 1234567891,
        },
        {
            type: MessageType.error,
            title: "错误",
            content: "无法连接到服务器",
            timestamp: 1234567892,
        },
    ];
    
    VerticalLayout {
        padding: 20px;
        spacing: 15px;
        
        Text {
            text: "通知中心 (" + messages.length + ")";
            font-size: 24px;
            font-weight: 700;
        }
        
        for msg[index] in messages : Rectangle {
            height: 100px;
            border-radius: 8px;
            border-width: 2px;
            
            // 根据消息类型设置样式
            background: {
                if msg.type == MessageType.info { #e3f2fd }
                else if msg.type == MessageType.warning { #fff3e0 }
                else if msg.type == MessageType.error { #ffebee }
                else { #e8f5e9 }
            };
            
            border-color: {
                if msg.type == MessageType.info { #2196f3 }
                else if msg.type == MessageType.warning { #ff9800 }
                else if msg.type == MessageType.error { #f44336 }
                else { #4caf50 }
            };
            
            HorizontalLayout {
                padding: 15px;
                spacing: 15px;
                
                // 图标
                Text {
                    text: {
                        if msg.type == MessageType.info { "ℹ️" }
                        else if msg.type == MessageType.warning { "⚠️" }
                        else if msg.type == MessageType.error { "❌" }
                        else { "✅" }
                    };
                    font-size: 32px;
                }
                
                // 内容
                VerticalLayout {
                    spacing: 5px;
                    
                    Text {
                        text: msg.title;
                        font-size: 16px;
                        font-weight: 700;
                    }
                    
                    Text {
                        text: msg.content;
                        font-size: 14px;
                        color: #666;
                    }
                }
            }
        }
    }
}
```

### 12.9 嵌套结构体

```slint
export struct Address {
    street: string,
    city: string,
    zip-code: string,
    country: string,
}

export struct Company {
    name: string,
    address: Address,
    employee-count: int,
}

export struct Employee {
    id: int,
    name: string,
    email: string,
    company: Company,
    is-active: bool,
}

export component EmployeeCard inherits Rectangle {
    preferred-width: 400px;
    preferred-height: 300px;
    
    in property <Employee> employee: {
        id: 1,
        name: "张三",
        email: "zhangsan@example.com",
        company: {
            name: "科技公司",
            address: {
                street: "中关村大街1号",
                city: "北京",
                zip-code: "100000",
                country: "中国",
            },
            employee-count: 500,
        },
        is-active: true,
    };
    
    background: white;
    border-radius: 10px;
    drop-shadow-blur: 10px;
    drop-shadow-color: #00000020;
    
    VerticalLayout {
        padding: 20px;
        spacing: 15px;
        
        // 员工信息
        HorizontalLayout {
            spacing: 10px;
            
            Rectangle {
                width: 60px;
                height: 60px;
                border-radius: 30px;
                background: #3498db;
                
                Text {
                    text: employee.name.charAt(0);  // 首字母
                    color: white;
                    font-size: 24px;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }
            
            VerticalLayout {
                spacing: 5px;
                
                Text {
                    text: employee.name;
                    font-size: 20px;
                    font-weight: 700;
                }
                
                Text {
                    text: employee.email;
                    font-size: 14px;
                    color: #7f8c8d;
                }
            }
        }
        
        // 分隔线
        Rectangle {
            height: 1px;
            background: #ecf0f1;
        }
        
        // 公司信息
        VerticalLayout {
            spacing: 8px;
            
            Text {
                text: "公司: " + employee.company.name;
                font-size: 16px;
            }
            
            Text {
                text: "地址: " + employee.company.address.city + ", " + 
                      employee.company.address.street;
                font-size: 14px;
                color: #95a5a6;
            }
            
            Text {
                text: "员工数: " + employee.company.employee-count;
                font-size: 14px;
                color: #95a5a6;
            }
        }
        
        // 状态标签
        Rectangle {
            width: 80px;
            height: 30px;
            border-radius: 15px;
            background: employee.is-active ? #2ecc71 : #e74c3c;
            
            Text {
                text: employee.is-active ? "在职" : "离职";
                color: white;
                horizontal-alignment: center;
                vertical-alignment: center;
            }
        }
    }
}
```

### 12.10 坑点与最佳实践

#### 坑点 1: 枚举比较必须使用完整名称

```slint
export enum Color {
    red,
    green,
    blue,
}

export component Example {
    property <Color> current-color: Color.red;
    
    // ❌ 错误: 在比较时不能省略枚举名
    // property <bool> is-red: current-color == red;
    
    // ✅ 正确: 必须使用完整名称
    property <bool> is-red: current-color == Color.red;
}
```

#### 坑点 2: 结构体字段访问越界

```slint
export struct Person {
    name: string,
    age: int,
}

export component Example {
    property <[Person]> people: [
        { name: "Alice", age: 30 },
    ];
    
    // ❌ 危险: 访问不存在的索引不会报错,返回默认值
    property <string> bad-name: people[10].name;  // 返回 ""
    
    // ✅ 最佳实践: 先检查长度
    property <string> safe-name: {
        if people.length > 0 {
            people[0].name
        } else {
            "No data"
        }
    };
}
```

#### 坑点 3: 结构体字段名与保留字冲突

```slint
// ❌ 避免使用保留字作为字段名
// export struct Bad {
//     property: string,  // 'property' 是保留字
//     function: string,  // 'function' 是保留字
// }

// ✅ 使用短横线命名
export struct Good {
    item-property: string,
    callback-function: string,
}
```

#### 最佳实践汇总

1. **命名规范**:
   - 结构体名使用 PascalCase: `UserProfile`
   - 枚举名使用 PascalCase: `AppState`
   - 枚举值使用 kebab-case: `dark-mode`
   - 字段名使用 kebab-case: `user-name`

2. **类型选择**:
   - 少量固定值 → 使用枚举
   - 数据组合 → 使用结构体
   - 简单数据 → 使用匿名结构体
   - 复用性高的数据 → 定义命名结构体

3. **导出策略**:
   ```slint
   // 需要从原生代码访问的类型必须导出
   export struct ApiResponse {
       status: int,
       message: string,
   }
   
   // 内部使用的类型不需要导出
   struct InternalConfig {
       debug-mode: bool,
   }
   ```

4. **默认值处理**:
   ```slint
   export struct Config {
       theme: string,
       font-size: int,
   }
   
   export component App {
       // 提供有意义的默认值
       property <Config> config: {
           theme: "light",
           font-size: 14,
       };
   }
   ```

---

## 13. 常见陷阱与最佳实践

### 13.1 性能陷阱

#### 陷阱 1: 过度使用 changed 回调

```slint
// ❌ 不好: 使用 changed 回调
export component Bad {
    in-out property <int> input;
    property <int> output;
    
    changed input => {
        output = input * 2;
    }
}

// ✅ 好: 使用声明式绑定
export component Good {
    in-out property <int> input;
    property <int> output: input * 2;  // 自动响应,性能更好
}
```

#### 陷阱 2: 频繁创建/销毁条件元素

```slint
// ❌ 性能差: 频繁切换会销毁/重建元素
export component Bad {
    in-out property <bool> show-panel: false;
    
    if show-panel : ComplexPanel {
        // 大量子元素...
    }
}

// ✅ 性能好: 使用 visible 属性
export component Good {
    in-out property <bool> show-panel: false;
    
    ComplexPanel {
        visible: show-panel;  // 只是隐藏,不销毁
        // 大量子元素...
    }
}
```

**何时使用 if vs visible**:
- 使用 `if`: 元素很少显示,或初始化成本低
- 使用 `visible`: 频繁切换,或初始化成本高

#### 陷阱 3: 在循环中创建大量动画

```slint
// ❌ 性能问题: 1000个动画同时运行
export component Bad {
    VerticalLayout {
        for i in 1000 : Rectangle {
            width: 100px;
            height: 2px;
            animate width { duration: 1s; }  // 1000个动画!
        }
    }
}

// ✅ 优化: 只动画可见元素或使用统一动画
export component Good {
    property <float> animation-progress: 0;
    
    VerticalLayout {
        for i in 1000 : Rectangle {
            width: 100px * animation-progress;  // 共享一个动画状态
            height: 2px;
        }
    }
    
    animate animation-progress { duration: 1s; }
}
```

### 13.2 响应式陷阱

#### 陷阱 4: 循环依赖

```slint
// ❌ 错误: 创建循环依赖
export component Bad {
    property <int> a: b + 1;
    property <int> b: a + 1;  // 循环!
}

// ✅ 正确: 避免循环依赖
export component Good {
    in-out property <int> source: 0;
    property <int> a: source + 1;
    property <int> b: source + 2;
}
```

#### 陷阱 5: 双向绑定陷阱

```slint
// ❌ 错误: 不能同时有绑定和双向绑定
export component Bad {
    property <int> value: 10;  // 有绑定
    
    Slider {
        value <=> root.value;  // ❌ 错误!已有绑定的属性不能双向绑定
    }
}

// ✅ 正确: 移除绑定或使用 in-out
export component Good {
    in-out property <int> value: 10;  // 使用 in-out
    
    Slider {
        value <=> root.value;  // ✅ OK
    }
}
```

### 13.3 布局陷阱

#### 陷阱 6: 忘记设置布局约束

```slint
// ❌ 问题: 元素可能尺寸为 0
export component Bad {
    HorizontalLayout {
        Rectangle {
            background: red;
            // 没有设置 width/min-width/preferred-width
        }
    }
}

// ✅ 正确: 明确设置约束
export component Good {
    HorizontalLayout {
        Rectangle {
            background: red;
            min-width: 50px;  // 至少50px
            horizontal-stretch: 1;  // 可以拉伸
        }
    }
}
```

#### 陷阱 7: 百分比单位误用

```slint
// ❌ 错误: 不能在所有地方使用百分比简写
export component Bad {
    Rectangle {
        x: 50%;  // ❌ 错误! x 不支持百分比简写
    }
}

// ✅ 正确: 使用完整表达式
export component Good {
    Rectangle {
        x: parent.width * 50%;  // ✅ OK
        width: 50%;  // ✅ OK: width 支持简写
    }
}
```

### 13.4 作用域陷阱

#### 陷阱 8: 混淆 self 和 root

```slint
export component Example inherits Rectangle {
    property <int> root-value: 10;
    
    Rectangle {
        property <int> local-value: 20;
        
        Text {
            // root 总是指向最外层组件
            text: "Root: " + root.root-value;  // ✅ 10
            
            // self 指向当前元素
            // text: "Local: " + self.local-value;  // ❌ 错误! Text 没有 local-value
        }
    }
}
```

#### 陷阱 9: 元素命名与属性冲突

```slint
// ❌ 容易混淆
export component Bad {
    property <int> value: 10;
    
    value := Rectangle {  // 元素名与属性名相同!
        width: 100px;
    }
    
    Text {
        // 访问属性还是元素?
        text: value;  // 这是什么?
    }
}

// ✅ 清晰命名
export component Good {
    property <int> current-value: 10;
    
    value-display := Rectangle {
        width: 100px;
    }
    
    Text {
        text: current-value;  // 清晰明确
    }
}
```

### 13.5 状态管理最佳实践

#### 单一数据源原则

```slint
// ❌ 不好: 重复的状态
export component Bad {
    in-out property <bool> is-loading: false;
    in-out property <bool> is-ready: false;
    in-out property <bool> has-error: false;
    
    // 状态不同步的风险
}

// ✅ 好: 单一状态源
export enum LoadState {
    idle,
    loading,
    success,
    error,
}

export component Good {
    in-out property <LoadState> state: LoadState.idle;
    
    // 派生状态
    property <bool> is-loading: state == LoadState.loading;
    property <bool> is-ready: state == LoadState.success;
    property <bool> has-error: state == LoadState.error;
}
```

#### 提升状态

```slint
// ❌ 不好: 状态分散
export component Bad inherits Window {
    VerticalLayout {
        Component1 {
            in-out property <string> shared-data;
        }
        Component2 {
            in-out property <string> shared-data;  // 重复!
        }
    }
}

// ✅ 好: 状态提升到共同父组件
export component Good inherits Window {
    in-out property <string> shared-data: "";
    
    VerticalLayout {
        Component1 {
            data <=> root.shared-data;
        }
        Component2 {
            data <=> root.shared-data;
        }
    }
}

component Component1 {
    in-out property <string> data;
}

component Component2 {
    in-out property <string> data;
}
```

### 13.6 代码组织最佳实践

#### 文件结构

```
project/
├── ui/
│   ├── app.slint              # 主入口
│   ├── components/            # 可复用组件
│   │   ├── button.slint
│   │   ├── card.slint
│   │   └── dialog.slint
│   ├── screens/               # 页面
│   │   ├── login.slint
│   │   ├── dashboard.slint
│   │   └── settings.slint
│   ├── globals/               # 全局单例
│   │   ├── theme.slint
│   │   └── app-state.slint
│   └── types/                 # 类型定义
│       ├── models.slint
│       └── enums.slint
└── src/
    └── main.rs
```

#### 组件拆分原则

```slint
// ❌ 不好: 单个文件太大
export component BigComponent inherits Window {
    // 500 行代码...
}

// ✅ 好: 拆分为小组件
// app.slint
import { Header } from "components/header.slint";
import { Sidebar } from "components/sidebar.slint";
import { MainContent } from "components/main-content.slint";

export component App inherits Window {
    VerticalLayout {
        Header { }
        HorizontalLayout {
            Sidebar { }
            MainContent { }
        }
    }
}
```

### 13.7 调试技巧

#### 使用 debug 函数

```slint
export component Example {
    callback test-callback(int) -> int;
    
    test-callback(value) => {
        debug("Input value:", value);
        
        let result = value * 2;
        debug("Calculated result:", result);
        
        return result;
    }
}
```

#### 属性监控

```slint
export component Example {
    in-out property <int> counter: 0;
    
    // 监控属性变化
    changed counter => {
        debug("Counter changed to:", self.counter);
    }
    
    callback increment;
    increment => {
        counter += 1;
        debug("After increment:", counter);
    }
}
```

#### 条件断点

```slint
export component Example {
    in-out property <int> value;
    
    changed value => {
        if self.value > 100 {
            debug("⚠️ Value exceeded 100:", self.value);
        }
    }
}
```

### 13.8 性能优化检查清单

✅ **做**:
- 优先使用声明式绑定而非回调
- 使用 `pure` 标记纯函数
- 合理使用 `visible` 而非 `if`
- 避免在循环中创建过多动画
- 使用适当的布局而非绝对定位
- 缓存计算结果(通过属性绑定)

❌ **不做**:
- 过度使用 `changed` 回调
- 创建循环依赖
- 在大型列表中使用复杂元素
- 频繁创建/销毁条件元素
- 在回调中进行耗时操作
- 过度嵌套布局

### 13.9 可访问性最佳实践

```slint
export component AccessibleButton inherits Rectangle {
    // 提供有意义的文本
    in property <string> text: "Button";
    
    // 提供可访问性信息(未来功能)
    // accessible-role: button;
    // accessible-label: text;
    
    // 确保足够的对比度
    background: #3498db;
    
    Text {
        text: root.text;
        color: white;  // 确保与背景有良好对比度
        font-size: 16px;  // 最小字体大小
    }
    
    // 提供视觉反馈
    touch := TouchArea {
        mouse-cursor: pointer;
    }
    
    states [
        hover when touch.has-hover: {
            background: #2980b9;
        }
        pressed when touch.pressed: {
            background: #21618c;
        }
    ]
}
```

### 13.10 总结 - 学习路径

#### 初级阶段
1. ✅ 掌握基本语法(元素、属性、表达式)
2. ✅ 学会使用内置元素和标准 Widgets
3. ✅ 理解布局系统(HorizontalLayout、VerticalLayout、GridLayout)
4. ✅ 创建简单的自定义组件

#### 中级阶段
5. ✅ 掌握属性系统(in、out、in-out)
6. ✅ 学会使用回调和函数
7. ✅ 理解数据模型和数组操作
8. ✅ 使用动画和状态管理
9. ✅ 与原生代码交互(Rust/C++)

#### 高级阶段
10. ✅ 优化性能和避免常见陷阱
11. ✅ 构建复杂的状态机
12. ✅ 设计可复用的组件库
13. ✅ 实现响应式和自适应布局
14. ✅ 掌握高级动画和过渡效果

#### 实战项目建议
- **入门**: 计数器、待办事项列表
- **进阶**: 天气应用、音乐播放器
- **高级**: 聊天应用、数据仪表板、游戏 UI

---

## 附录 A: 快速参考

### 类型对照表

| Slint 类型 | 描述 | 默认值 | 示例 |
|-----------|------|--------|------|
| `int` | 32位整数 | 0 | `42` |
| `float` | 32位浮点数 | 0.0 | `3.14` |
| `bool` | 布尔值 | false | `true` |
| `string` | 字符串 | "" | `"Hello"` |
| `color` | 颜色 | transparent | `#ff0000` |
| `length` | 长度 | 0px | `100px` |
| `duration` | 时长 | 0ms | `500ms` |
| `angle` | 角度 | 0deg | `45deg` |

### 常用元素速查

| 元素 | 用途 | 关键属性 |
|------|------|----------|
| `Rectangle` | 矩形容器 | `background`, `border-*` |
| `Text` | 文本显示 | `text`, `color`, `font-*` |
| `Image` | 图片显示 | `source`, `image-fit` |
| `TouchArea` | 交互区域 | `clicked`, `pressed` |
| `HorizontalLayout` | 水平布局 | `spacing`, `alignment` |
| `VerticalLayout` | 垂直布局 | `spacing`, `alignment` |
| `GridLayout` | 网格布局 | `spacing`, `Row` |
| `FocusScope` | 焦点管理 | `has-focus`, `enabled` |

### 布局属性速查

| 属性 | 适用元素 | 说明 |
|------|----------|------|
| `horizontal-stretch` | 布局子元素 | 水平拉伸因子(默认1) |
| `vertical-stretch` | 布局子元素 | 垂直拉伸因子(默认1) |
| `min-width` | 所有元素 | 最小宽度 |
| `max-width` | 所有元素 | 最大宽度 |
| `preferred-width` | 所有元素 | 首选宽度 |
| `alignment` | 布局 | 对齐方式 |
| `spacing` | 布局 | 元素间距 |
| `padding` | 布局 | 内边距 |

### 动画缓动函数速查

| 函数 | 效果 | 适用场景 |
|------|------|----------|
| `linear` | 线性,匀速 | 进度条、加载动画 |
| `ease` | 慢-快-慢 | 通用动画 |
| `ease-in` | 加速 | 元素消失 |
| `ease-out` | 减速 | 元素出现 |
| `ease-in-out` | 两端慢 | 位置移动 |
| `ease-out-bounce` | 弹跳 | 按钮反馈 |
| `ease-out-elastic` | 弹性 | 特殊效果 |

---

## 附录 B: 常见问题 FAQ

### Q1: 如何从原生代码更新 UI?

**A**: 通过属性和回调:

```slint
// ui.slint
export component App inherits Window {
    in-out property <string> status-text;
    callback refresh-data();
}
```

```rust
// Rust
let app = App::new().unwrap();

// 设置属性
app.set_status_text("Loading...".into());

// 设置回调
app.on_refresh_data({
    let app_weak = app.as_weak();
    move || {
        // 异步操作...
        let app = app_weak.upgrade().unwrap();
        app.set_status_text("Done!".into());
    }
});
```

### Q2: 如何处理大型列表?

**A**: 使用 `for` 循环和模型:

```slint
export component ListView inherits Window {
    in-out property <[{text: string}]> items;
    
    VerticalLayout {
        for item[index] in items : Rectangle {
            height: 40px;
            Text { text: item.text; }
        }
    }
}
```

```rust
// Rust - 使用 VecModel
let items = Rc::new(VecModel::from(vec![
    Item { text: "Item 1".into() },
    // ...
]));
app.set_items(items.into());
```

### Q3: 如何实现暗黑模式?

**A**: 使用全局单例:

```slint
global Theme {
    in-out property <bool> dark-mode: false;
    in-out property <color> background: dark-mode ? #2c3e50 : #ecf0f1;
    in-out property <color> text: dark-mode ? #ecf0f1 : #2c3e50;
}

export component App inherits Window {
    background: Theme.background;
    
    Text {
        text: "Hello";
        color: Theme.text;
    }
    
    Button {
        text: "Toggle Theme";
        clicked => {
            Theme.dark-mode = !Theme.dark-mode;
        }
    }
}
```

### Q4: 如何实现路由/页面导航?

**A**: 使用枚举状态:

```slint
export enum Page {
    home,
    settings,
    about,
}

export component App inherits Window {
    in-out property <Page> current-page: Page.home;
    
    if current-page == Page.home : HomePage { }
    if current-page == Page.settings : SettingsPage { }
    if current-page == Page.about : AboutPage { }
    
    // 导航栏
    HorizontalLayout {
        Button {
            text: "Home";
            clicked => { current-page = Page.home; }
        }
        Button {
            text: "Settings";
            clicked => { current-page = Page.settings; }
        }
    }
}
```

### Q5: 如何处理表单验证?

**A**: 使用派生属性:

```slint
export component LoginForm inherits Rectangle {
    in-out property <string> email;
    in-out property <string> password;
    
    // 验证规则
    property <bool> email-valid: email.length > 0 && email.contains("@");
    property <bool> password-valid: password.length >= 6;
    property <bool> form-valid: email-valid && password-valid;
    
    callback submit();
    
    VerticalLayout {
        LineEdit {
            text <=> email;
            placeholder-text: "Email";
        }
        
        if !email-valid && email.length > 0 : Text {
            text: "Invalid email";
            color: red;
        }
        
        LineEdit {
            text <=> password;
            input-type: password;
        }
        
        if !password-valid && password.length > 0 : Text {
            text: "Password must be at least 6 characters";
            color: red;
        }
        
        Button {
            text: "Login";
            enabled: form-valid;
            clicked => {
                if form-valid {
                    root.submit();
                }
            }
        }
    }
}
```

### Q6: 如何处理异步操作?

**A**: 在原生代码中处理,通过回调通知 UI:

```slint
export component App inherits Window {
    in-out property <bool> loading: false;
    in-out property <string> result;
    
    callback fetch-data();
    
    Button {
        text: loading ? "Loading..." : "Fetch Data";
        enabled: !loading;
        clicked => { root.fetch-data(); }
    }
    
    Text {
        text: result;
    }
}
```

```rust
// Rust
app.on_fetch_data({
    let app_weak = app.as_weak();
    move || {
        let app = app_weak.upgrade().unwrap();
        app.set_loading(true);
        
        // 异步操作
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            
            // 更新 UI (需要在主线程)
            slint::invoke_from_event_loop(move || {
                let app = app_weak.upgrade().unwrap();
                app.set_result("Data loaded!".into());
                app.set_loading(false);
            }).unwrap();
        });
    }
});
```

### Q7: 如何优化大型应用性能?

**A**: 遵循以下原则:

1. **懒加载**: 使用 `if` 按需显示组件
2. **虚拟滚动**: 对于长列表,只渲染可见项
3. **避免过度绑定**: 减少复杂的属性依赖链
4. **使用 pure 函数**: 标记纯函数以便优化
5. **批量更新**: 在原生代码中批量修改数据

```slint
// 懒加载示例
export component App {
    in-out property <bool> show-heavy-component: false;
    
    Button {
        text: "Show";
        clicked => { show-heavy-component = true; }
    }
    
    // 只在需要时创建
    if show-heavy-component : HeavyComponent { }
}
```

### Q8: 如何处理国际化(i18n)?

**A**: 使用全局单例存储翻译:

```slint
global Translations {
    in-out property <string> language: "en";
    
    public pure function get-text(key: string) -> string {
        if language == "zh" {
            if key == "hello" { return "你好"; }
            if key == "goodbye" { return "再见"; }
        } else {
            if key == "hello" { return "Hello"; }
            if key == "goodbye" { return "Goodbye"; }
        }
        return key;
    }
}

export component App {
    Text {
        text: Translations.get-text("hello");
    }
    
    Button {
        text: "切换语言";
        clicked => {
            Translations.language = 
                Translations.language == "en" ? "zh" : "en";
        }
    }
}
```

更好的方案是在 Rust 中处理:

```rust
app.global::<Translations>().on_get_text(|key| {
    match &*key {
        "hello" => get_translation("hello").into(),
        "goodbye" => get_translation("goodbye").into(),
        _ => key.clone(),
    }
});
```

### Q9: 如何创建可复用的组件库?

**A**: 项目结构示例:

```
my-component-library/
├── ui/
│   ├── components/
│   │   ├── index.slint         # 导出所有组件
│   │   ├── button.slint
│   │   ├── card.slint
│   │   └── dialog.slint
│   └── themes/
│       └── default.slint
└── Cargo.toml
```

```slint
// components/index.slint
export { Button } from "button.slint";
export { Card } from "card.slint";
export { Dialog } from "dialog.slint";
```

使用库:

```slint
import { Button, Card } from "@mylib/components/index.slint";

export component App {
    Card {
        Button { text: "Click me"; }
    }
}
```

### Q10: 如何调试 Slint 应用?

**A**: 调试技巧:

1. **使用 debug() 函数**:
```slint
callback test;
test => {
    debug("Test called");
    debug("Value:", some-property);
}
```

2. **监控属性变化**:
```slint
in-out property <int> value;
changed value => {
    debug("Value changed to:", self.value);
}
```

3. **使用 slint-viewer 实时预览**:
```bash
slint-viewer ui/app.slint
```

4. **Rust 端调试**:
```rust
// 打印属性值
println!("Status: {}", app.get_status());

// 断点调试
app.on_some_callback(|| {
    // 设置断点
    println!("Callback triggered");
});
```

5. **VS Code 扩展**: 安装 Slint 扩展获得:
   - 语法高亮
   - 实时预览
   - 代码补全
   - 错误检查

---

## 附录 C: 实战项目示例

### 完整的计数器应用

```slint
// counter_app.slint
export component CounterApp inherits Window {
    preferred-width: 400px;
    preferred-height: 300px;
    title: "Counter App";
    
    in-out property <int> counter: 0;
    
    callback increment;
    callback decrement;
    callback reset;
    
    increment => { counter += 1; }
    decrement => { counter -= 1; }
    reset => { counter = 0; }
    
    background: #ecf0f1;
    
    VerticalLayout {
        padding: 40px;
        spacing: 30px;
        alignment: center;
        
        // 标题
        Text {
            text: "Counter App";
            font-size: 32px;
            font-weight: 700;
            color: #2c3e50;
            horizontal-alignment: center;
        }
        
        // 计数显示
        Rectangle {
            preferred-height: 120px;
            background: white;
            border-radius: 15px;
            drop-shadow-blur: 10px;
            drop-shadow-color: #00000020;
            
            Text {
                text: counter;
                font-size: 64px;
                font-weight: 700;
                color: counter >= 0 ? #27ae60 : #e74c3c;
                horizontal-alignment: center;
                vertical-alignment: center;
            }
        }
        
        // 按钮组
        HorizontalLayout {
            spacing: 15px;
            alignment: center;
            
            Rectangle {
                width: 80px;
                height: 80px;
                border-radius: 40px;
                background: touch-minus.pressed ? #c0392b : #e74c3c;
                
                animate background { duration: 150ms; }
                
                Text {
                    text: "−";
                    font-size: 48px;
                    color: white;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
                
                touch-minus := TouchArea {
                    clicked => { root.decrement(); }
                }
            }
            
            Rectangle {
                width: 80px;
                height: 80px;
                border-radius: 40px;
                background: touch-reset.pressed ? #7f8c8d : #95a5a6;
                
                animate background { duration: 150ms; }
                
                Text {
                    text: "↻";
                    font-size: 36px;
                    color: white;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
                
                touch-reset := TouchArea {
                    clicked => { root.reset(); }
                }
            }
            
            Rectangle {
                width: 80px;
                height: 80px;
                border-radius: 40px;
                background: touch-plus.pressed ? #229954 : #27ae60;
                
                animate background { duration: 150ms; }
                
                Text {
                    text: "+";
                    font-size: 48px;
                    color: white;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
                
                touch-plus := TouchArea {
                    clicked => { root.increment(); }
                }
            }
        }
        
        // 统计信息
        Text {
            text: "Total clicks: " + (counter >= 0 ? counter : -counter);
            font-size: 16px;
            color: #7f8c8d;
            horizontal-alignment: center;
        }
    }
}
```

### 完整的待办事项应用

```slint
// todo_app.slint
export struct TodoItem {
    id: int,
    text: string,
    completed: bool,
}

export enum Filter {
    all,
    active,
    completed,
}

export component TodoApp inherits Window {
    preferred-width: 600px;
    preferred-height: 700px;
    title: "Todo List";
    
    in-out property <[TodoItem]> todos;
    in-out property <Filter> current-filter: Filter.all;
    in-out property <string> input-text;
    
    callback add-todo(string);
    callback toggle-todo(int);
    callback delete-todo(int);
    callback clear-completed();
    
    // 过滤后的待办列表
    property <[TodoItem]> filtered-todos: {
        if current-filter == Filter.active {
            // 只显示未完成
            // 注意:这里简化处理,实际需要在 Rust 端过滤
            todos
        } else if current-filter == Filter.completed {
            // 只显示已完成
            todos
        } else {
            todos
        }
    };
    
    property <int> active-count: {
        let count = 0;
        // 实际计数需要在 Rust 端
        count
    };
    
    background: #f5f5f5;
    
    VerticalLayout {
        padding: 0px;
        
        // 头部
        Rectangle {
            height: 100px;
            background: #3498db;
            drop-shadow-blur: 5px;
            drop-shadow-color: #00000030;
            
            VerticalLayout {
                padding: 20px;
                
                Text {
                    text: "Todo List";
                    font-size: 32px;
                    font-weight: 700;
                    color: white;
                }
            }
        }
        
        // 输入区域
        Rectangle {
            height: 80px;
            background: white;
            
            HorizontalLayout {
                padding: 20px;
                spacing: 15px;
                
                Rectangle {
                    background: #ecf0f1;
                    border-radius: 8px;
                    horizontal-stretch: 1;
                    
                    LineEdit {
                        text <=> input-text;
                        placeholder-text: "What needs to be done?";
                        font-size: 16px;
                        
                        accepted => {
                            if input-text.length > 0 {
                                root.add-todo(input-text);
                                input-text = "";
                            }
                        }
                    }
                }
                
                Rectangle {
                    width: 100px;
                    height: 50px;
                    border-radius: 8px;
                    background: add-touch.pressed ? #229954 : #27ae60;
                    
                    animate background { duration: 150ms; }
                    
                    Text {
                        text: "Add";
                        font-size: 16px;
                        font-weight: 700;
                        color: white;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                    
                    add-touch := TouchArea {
                        clicked => {
                            if input-text.length > 0 {
                                root.add-todo(input-text);
                                input-text = "";
                            }
                        }
                    }
                }
            }
        }
        
        // 列表区域
        Rectangle {
            background: white;
            vertical-stretch: 1;
            
            VerticalLayout {
                padding: 0px;
                spacing: 0px;
                
                for item[index] in todos : Rectangle {
                    height: 60px;
                    background: item-touch.has-hover ? #f8f9fa : white;
                    
                    animate background { duration: 150ms; }
                    
                    HorizontalLayout {
                        padding-left: 20px;
                        padding-right: 20px;
                        spacing: 15px;
                        
                        // 复选框
                        Rectangle {
                            width: 24px;
                            height: 24px;
                            border-radius: 4px;
                            border-width: 2px;
                            border-color: item.completed ? #27ae60 : #bdc3c7;
                            background: item.completed ? #27ae60 : transparent;
                            
                            animate background, border-color {
                                duration: 200ms;
                            }
                            
                            if item.completed : Text {
                                text: "✓";
                                color: white;
                                font-size: 16px;
                                font-weight: 700;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                            
                            TouchArea {
                                clicked => {
                                    root.toggle-todo(item.id);
                                }
                            }
                        }
                        
                        // 文本
                        Text {
                            text: item.text;
                            font-size: 16px;
                            color: item.completed ? #95a5a6 : #2c3e50;
                            horizontal-stretch: 1;
                            vertical-alignment: center;
                        }
                        
                        // 删除按钮
                        Rectangle {
                            width: 30px;
                            height: 30px;
                            border-radius: 15px;
                            background: del-touch.has-hover ? #e74c3c : transparent;
                            
                            animate background { duration: 150ms; }
                            
                            Text {
                                text: "×";
                                font-size: 24px;
                                color: del-touch.has-hover ? white : #95a5a6;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                            
                            del-touch := TouchArea {
                                clicked => {
                                    root.delete-todo(item.id);
                                }
                            }
                        }
                    }
                    
                    item-touch := TouchArea { }
                    
                    // 分隔线
                    Rectangle {
                        y: parent.height - 1px;
                        height: 1px;
                        background: #ecf0f1;
                    }
                }
            }
        }
        
        // 底部工具栏
        Rectangle {
            height: 60px;
            background: white;
            border-width: 1px;
            border-color: #e1e4e8;
            
            HorizontalLayout {
                padding: 15px 20px;
                spacing: 20px;
                
                Text {
                    text: active-count + " items left";
                    font-size: 14px;
                    color: #7f8c8d;
                    vertical-alignment: center;
                }
                
                Rectangle { horizontal-stretch: 1; }
                
                // 过滤按钮
                HorizontalLayout {
                    spacing: 10px;
                    
                    Rectangle {
                        width: 60px;
                        height: 30px;
                        border-radius: 4px;
                        background: current-filter == Filter.all ? #3498db : transparent;
                        
                        Text {
                            text: "All";
                            color: current-filter == Filter.all ? white : #7f8c8d;
                            font-size: 12px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        
                        TouchArea {
                            clicked => { current-filter = Filter.all; }
                        }
                    }
                    
                    Rectangle {
                        width: 60px;
                        height: 30px;
                        border-radius: 4px;
                        background: current-filter == Filter.active ? #3498db : transparent;
                        
                        Text {
                            text: "Active";
                            color: current-filter == Filter.active ? white : #7f8c8d;
                            font-size: 12px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        
                        TouchArea {
                            clicked => { current-filter = Filter.active; }
                        }
                    }
                    
                    Rectangle {
                        width: 80px;
                        height: 30px;
                        border-radius: 4px;
                        background: current-filter == Filter.completed ? #3498db : transparent;
                        
                        Text {
                            text: "Completed";
                            color: current-filter == Filter.completed ? white : #7f8c8d;
                            font-size: 12px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        
                        TouchArea {
                            clicked => { current-filter = Filter.completed; }
                        }
                    }
                }
            }
        }
    }
}
```

---

## 结语

恭喜你完成了这个全面的 Slint UI 框架教程!你现在已经掌握了:

✅ **基础知识**: 文件结构、组件、属性、表达式
✅ **布局系统**: 定位、布局容器、响应式设计  
✅ **高级特性**: 动画、状态、全局单例、数据模型
✅ **最佳实践**: 性能优化、代码组织、调试技巧
✅ **实战经验**: 完整的示例项目

### 继续学习资源

- 📖 **官方文档**: https://slint.dev/docs
- 💬 **社区论坛**: https://github.com/slint-ui/slint/discussions
- 📺 **示例项目**: https://github.com/slint-ui/slint/tree/master/examples
- 🎓 **在线教程**: https://slint.dev/tutorials

### 下一步建议

1. 尝试构建自己的小项目
2. 阅读 Slint 源码中的示例
3. 参与社区讨论和贡献
4. 探索与其他框架的集成(如 Tauri)

记住:**实践是最好的老师**。开始编码,遇到问题时参考本教程,你会越来越熟练!

祝你在 Slint 开发之旅中一切顺利! 🚀