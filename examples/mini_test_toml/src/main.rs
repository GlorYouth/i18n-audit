use rust_i18n::t;

// 初始化翻译
rust_i18n::i18n!("../locales");

fn main() {
    // 设置当前语言
    rust_i18n::set_locale("zh-CN");
    
    // 示例 1: 使用字面量键
    println!("问候: {}", t!("greetings.hello"));
    println!("再见: {}", t!("greetings.goodbye"));
    
    // 示例 2: 带参数的翻译
    println!("欢迎信息: {}", t!("user.welcome", name = "张三"));
    
    // 示例 3: 动态键
    let key = "dynamic.key";
    println!("动态键: {}", t!(key));
    
    // 示例 4: 使用 format! 构建的动态键
    let section = "section";
    let id = 123;
    // 注意：rust-i18n 需要字面量键，因此这里只是示范如何分析这类情况
    let formatted_key = format!("content.{}.item.{}", section, id);
    println!("格式化动态键: {}", t!("content.section.item.123")); // 正确用法，使用字面量
    println!("(假设的动态键实现): {}", formatted_key); // 这里只是打印，不是实际调用 t!()
    
    // 注意: 下面的翻译键在翻译文件中定义，但在代码中未使用
    // - unused.key1
    // - unused.key2
    // - unused.nested.key
} 