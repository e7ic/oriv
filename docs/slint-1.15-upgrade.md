# Slint 1.15 升级记录

> 升级日期：2026-02-07  
> 版本变更：1.14.1 → 1.15.0  
> 官方发布说明：https://slint.dev/blog/slint-1.15-released

---

## 主要新特性

### 1. 动态 GridLayout

GridLayout 现在支持完全动态布局：

- 使用 `for` 循环动态生成行和单元格
- 为 `col` 和 `row` 属性设置任意绑定
- 使用 `if` 条件显示/隐藏单元格

```slint
GridLayout {
    spacing: 16px;
    for action[index] in actions: ActionButton {
        col: index.mod(2);
        row: index / 2;
        icon: action.icon;
        text: action.name;
    }
}
```

**项目应用场景**：可用于日志表格、规则列表等需要表格对齐的场景。

---

### 2. 结构体字段双向绑定

双向绑定 (`<=>`) 现在可以直接用于 struct 字段：

```slint
struct FooData { title: string, value: float }

export component Foo {
    in-out property <FooData> data;
    LineEdit { text <=> data.title; }
    Slider { value <=> data.value; }
}
```

**项目应用场景**：`ProxyRule`、`LogEntry`、`CertInfo` 等结构体的表单编辑。

---

### 3. OKLCH 颜色函数

新增颜色处理函数：

- `Colors.oklch(l, c, h)` - 创建 OKLCH 颜色
- `.to-oklch()` - 转换为 OKLCH

**项目应用场景**：主题颜色系统优化。

---

### 4. iOS/Android 支持增强

- `safe-area-insets` - 安全区域边距
- `virtual-keyboard-position` - 虚拟键盘位置
- `virtual-keyboard-size` - 虚拟键盘大小

**项目应用场景**：未来移动端支持。

---

### 5. 其他改进

| 改进项               | 说明                             |
| -------------------- | -------------------------------- |
| 软件渲染器 Path 支持 | 软件渲染器现在支持 Path 渲染     |
| BorrowMutError 修复  | 修复了软件渲染器的 panic         |
| Flickable 焦点优化   | 调整窗口大小时保持焦点元素可见   |
| 像素边界渲染         | 文本和图像在像素边界渲染，更清晰 |
| WGPU 28              | 升级到 WGPU 28                   |
| Live-Preview 修复    | 实时预览更加可靠                 |

---

## 迁移注意事项

本次升级为小版本升级，API 兼容，无需修改现有代码。

## 相关文件变更

- `Cargo.toml` - slint, i-slint-backend-winit, slint-build → 1.15.0
- `installer-bootstrap/Cargo.toml` - slint, slint-build → 1.15.0
