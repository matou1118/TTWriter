//! 中文排版预设：一键把「公文 / 论文 / 法律 / 通用」的版式套到当前文档。
//!
//! 预设本身就是命令序列，文件在 `presets/`，用 `wordcraft-cli run --script` 跑的是同一份，
//! 所以界面和命令行不会各改各的。字体只写 family 名：机器上没装 仿宋/黑体 时，
//! 渲染会回退（内置的阿里巴巴普惠体，或系统里别的中文字体），不会显示成方块。

use serde_json::Value;

use crate::WordApp;

/// 一份预设：界面上的名字 + 仓库里的命令脚本。
pub struct Preset {
    pub label: &'static str,
    pub json: &'static str,
}

pub const PRESETS: &[Preset] = &[
    Preset { label: "公文（国标 9704）", json: include_str!("../../../presets/zh-gongwen.json") },
    Preset { label: "论文", json: include_str!("../../../presets/zh-thesis.json") },
    Preset { label: "法律文书", json: include_str!("../../../presets/zh-legal.json") },
    Preset { label: "通用中文", json: include_str!("../../../presets/zh-general.json") },
];

/// 依次执行预设里的命令；第一条出错就停下并把原因报到状态栏。
///
/// shortcut: 每条命令是一步撤销，套一次预设要按几次撤销，等引擎有多步命令组时再合并。
pub fn apply(app: &mut WordApp, preset: &Preset) {
    // 界面字体里没有全角冒号和直角引号，写了就是空心方块（theme::tests::interface_symbols_have_glyphs）。
    let parsed: Value = match serde_json::from_str(preset.json) {
        Ok(v) => v,
        Err(e) => {
            app.status(format!("{} 预设解析失败（{e}）", preset.label));
            return;
        }
    };
    let Some(commands) = parsed.get("commands").and_then(Value::as_array) else {
        app.status(format!("{} 预设里没有 commands", preset.label));
        return;
    };
    for c in commands {
        let Some(id) = c.get("command").and_then(Value::as_str) else { continue };
        let params = c.get("params").cloned().unwrap_or(Value::Null);
        if let Err(e) = app.run(id, params) {
            app.status(format!("{} {id} 失败（{e}）", preset.label));
            return;
        }
    }
    app.status(format!("已套用 {} 版式", preset.label));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 四份预设都要能解析，且命令都在命令表里，改预设时界面不会静默失效。
    #[test]
    fn every_preset_parses_and_its_commands_exist() {
        let app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Default::default());
        for p in PRESETS {
            let v: Value = serde_json::from_str(p.json).unwrap_or_else(|e| panic!("{}: {e}", p.label));
            let commands = v.get("commands").and_then(Value::as_array).unwrap_or_else(|| panic!("{}: 没有 commands", p.label));
            assert!(!commands.is_empty(), "{}: commands 是空的", p.label);
            for c in commands {
                let id = c.get("command").and_then(Value::as_str).unwrap_or_else(|| panic!("{}: 命令缺 command 字段", p.label));
                assert!(app.session.registry.get(id).is_some(), "{}: 命令表里没有 {id}", p.label);
            }
        }
    }
}
