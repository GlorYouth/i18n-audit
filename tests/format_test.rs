use i18n_audit::{config::Config, actions};
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

fn run_format_test(
    file_a_name: &str,
    file_a_content: &str,
    file_b_name: &str,
    file_b_content: &str,
    master_keys: BTreeMap<String, ()>,
    expected_a_content: &str,
    expected_b_content: &str,
) {
    let dir = tempdir().unwrap();
    let locales_dir = dir.path().join("locales");
    fs::create_dir(&locales_dir).unwrap();

    let file_a_path = locales_dir.join(file_a_name);
    fs::write(&file_a_path, file_a_content).unwrap();

    let file_b_path = locales_dir.join(file_b_name);
    fs::write(&file_b_path, file_b_content).unwrap();

    let config = Config {
        project_path: dir.path().to_path_buf(),
        src_dir: "src".to_string(),
        locales_dir: "locales".to_string(),
        threshold: 20.0,
        ignore_pattern: None,
        ignore_todo_files: false,
        verbose: false,
    };

    actions::format_keys(&config, master_keys).unwrap();

    let result_a = fs::read_to_string(file_a_path).unwrap();
    let result_b = fs::read_to_string(file_b_path).unwrap();

    assert_eq!(result_a.trim(), expected_a_content.trim());
    assert_eq!(result_b.trim(), expected_b_content.trim());
}

#[test]
fn test_format_toml_files_alignment() {
    let en_content = r#"
common.ok = "OK"
common.cancel = "Cancel"
login.title = "Login"
"#;

    let zh_cn_content = r#"
common.ok = "好的"
login.title = "登录"
login.user_name = "用户名"
"#;

    let mut master_keys = BTreeMap::new();
    master_keys.insert("common.ok".to_string(), ());
    master_keys.insert("common.cancel".to_string(), ());
    master_keys.insert("login.title".to_string(), ());
    master_keys.insert("login.user_name".to_string(), ());

    let expected_en_content = r#"
[common]
cancel = "Cancel"
ok = "OK"

[login]
title = "Login"
user_name = ""
"#;

    let expected_zh_cn_content = r#"
[common]
cancel = ""
ok = "好的"

[login]
title = "登录"
user_name = "用户名"
"#;

    run_format_test(
        "en.toml",
        en_content,
        "zh-CN.toml",
        zh_cn_content,
        master_keys,
        expected_en_content,
        expected_zh_cn_content,
    );
}

#[test]
fn test_format_json_files_alignment() {
    let en_content = r#"{
  "common": {
    "ok": "OK",
    "cancel": "Cancel"
  },
  "login": {
    "title": "Login"
  }
}"#;

    let zh_cn_content = r#"{
  "common": {
    "ok": "好的"
  },
  "login": {
    "title": "登录",
    "user_name": "用户名"
  }
}"#;

    let mut master_keys = BTreeMap::new();
    master_keys.insert("common.ok".to_string(), ());
    master_keys.insert("common.cancel".to_string(), ());
    master_keys.insert("login.title".to_string(), ());
    master_keys.insert("login.user_name".to_string(), ());

    let expected_en_content = r#"{
  "common": {
    "cancel": "Cancel",
    "ok": "OK"
  },
  "login": {
    "title": "Login",
    "user_name": ""
  }
}"#;

    let expected_zh_cn_content = r#"{
  "common": {
    "cancel": "",
    "ok": "好的"
  },
  "login": {
    "title": "登录",
    "user_name": "用户名"
  }
}"#;

    run_format_test(
        "en.json",
        en_content,
        "zh-CN.json",
        zh_cn_content,
        master_keys,
        expected_en_content,
        expected_zh_cn_content,
    );
}

#[test]
fn test_format_yaml_files_alignment() {
    let en_content = r#"
common:
  ok: OK
  cancel: Cancel
login:
  title: Login
"#;

    let zh_cn_content = r#"
common:
  ok: 好的
login:
  title: 登录
  user_name: 用户名
"#;

    let mut master_keys = BTreeMap::new();
    master_keys.insert("common.ok".to_string(), ());
    master_keys.insert("common.cancel".to_string(), ());
    master_keys.insert("login.title".to_string(), ());
    master_keys.insert("login.user_name".to_string(), ());

    let expected_en_content = r#"
common:
  cancel: Cancel
  ok: OK
login:
  title: Login
  user_name: ''
"#;

    let expected_zh_cn_content = r#"
common:
  cancel: ''
  ok: 好的
login:
  title: 登录
  user_name: 用户名
"#;

    run_format_test(
        "en.yml",
        en_content,
        "zh-CN.yml",
        zh_cn_content,
        master_keys,
        expected_en_content,
        expected_zh_cn_content,
    );
} 