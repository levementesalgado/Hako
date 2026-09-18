use hako::parser::Parser;
use hako::ast::*;

// ===== Parser Tests =====

#[test]
fn test_parse_empty_program() {
    let mut p = Parser::new("");
    let prog = p.parse().unwrap();
    assert!(prog.boxes.is_empty());
    assert!(prog.flows.is_empty());
}

#[test]
fn test_parse_single_box() {
    let input = r#"
box serial {
    COM1 = 0x3F8
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    assert_eq!(prog.boxes.len(), 1);
    assert_eq!(prog.boxes[0].name, "serial");
    assert!(prog.boxes[0].extends.is_none());
    assert_eq!(prog.boxes[0].items.len(), 1);
}

#[test]
fn test_parse_box_with_extends() {
    let input = r#"
box vga::video {
    BASE = 0xB8000
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    assert_eq!(prog.boxes.len(), 1);
    assert_eq!(prog.boxes[0].name, "vga");
    assert_eq!(prog.boxes[0].extends, Some("video".to_string()));
}

#[test]
fn test_parse_box_constants() {
    let input = r#"
box test {
    PORT_A = 0x3F8
    PORT_B = 0x2F8
    SIZE = 1024
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    assert_eq!(prog.boxes[0].items.len(), 3);
    match &prog.boxes[0].items[0] {
        Item::Var { name, value } => {
            assert_eq!(name, "PORT_A");
            assert_eq!(value.as_deref(), Some("0x3F8"));
        }
        _ => panic!("expected Var"),
    }
}

#[test]
fn test_parse_auto_mode_function() {
    let input = r#"
box serial {
    config => default
    write_byte(b) => default
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    assert_eq!(prog.boxes[0].items.len(), 2);
    match &prog.boxes[0].items[0] {
        Item::Fn { name, params, mode, .. } => {
            assert_eq!(name, "config");
            assert!(params.is_empty());
            assert!(matches!(mode, FnMode::Auto));
        }
        _ => panic!("expected Fn"),
    }
    match &prog.boxes[0].items[1] {
        Item::Fn { name, params, mode, .. } => {
            assert_eq!(name, "write_byte");
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].0, "b");
            assert!(matches!(mode, FnMode::Auto));
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_block_function() {
    let input = r#"
box test {
    run {
        x = 10
        y = 20
    }
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { name, mode, body, .. } => {
            assert_eq!(name, "run");
            assert!(matches!(mode, FnMode::Block));
            assert_eq!(body.len(), 2);
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_raw_function() {
    let input = r#"
box test {
    asm_test raw {
        out(al, x)
        "mov eax, 1"
        in(eax, result)
    }
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { name, mode, body, .. } => {
            assert_eq!(name, "asm_test");
            assert!(matches!(mode, FnMode::Raw));
            assert_eq!(body.len(), 3);
            assert!(matches!(&body[0], Stmt::AsmOut { reg, var } if reg == "al" && var == "x"));
            assert!(matches!(&body[1], Stmt::AsmLine(s) if s == "mov eax, 1"));
            assert!(matches!(&body[2], Stmt::AsmIn { reg, var } if reg == "eax" && var == "result"));
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_flow() {
    let input = r#"
flow default {
    serial
    vga
    keyboard
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    assert_eq!(prog.flows.len(), 1);
    assert_eq!(prog.flows[0].name, "default");
    assert_eq!(prog.flows[0].steps, vec!["serial", "vga", "keyboard"]);
}

#[test]
fn test_parse_flow_with_arrows() {
    let input = r#"
flow init {
    serial -> vga -> keyboard
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    assert_eq!(prog.flows[0].steps, vec!["serial", "vga", "keyboard"]);
}

#[test]
fn test_parse_if_statement() {
    let input = r#"
box test {
    check {
        if x > 0 => {
            y = 1
        } else {
            y = 0
        }
    }
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { body, .. } => {
            assert_eq!(body.len(), 1);
            match &body[0] {
                Stmt::If { cond, then, r#else } => {
                    assert_eq!(cond, "x > 0");
                    assert_eq!(then.len(), 1);
                    assert_eq!(r#else.len(), 1);
                }
                _ => panic!("expected If"),
            }
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_loop_statement() {
    let input = r#"
box test {
    run {
        loop {
            x = x + 1
        }
    }
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { body, .. } => {
            assert!(matches!(&body[0], Stmt::Loop(_)));
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_for_statement() {
    let input = r#"
box test {
    run {
        for i in 0..10 {
            x = i
        }
    }
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { body, .. } => {
            match &body[0] {
                Stmt::For { var, iter, body } => {
                    assert_eq!(var, "i");
                    assert_eq!(iter, "0..10");
                    assert_eq!(body.len(), 1);
                }
                _ => panic!("expected For"),
            }
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_break_statement() {
    let input = r#"
box test {
    run {
        break
    }
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { body, .. } => {
            assert!(matches!(&body[0], Stmt::Break));
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_assignment() {
    let input = r#"
box test {
    run {
        x = 10 + 20
    }
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { body, .. } => {
            match &body[0] {
                Stmt::Assign { var, value } => {
                    assert_eq!(var, "x");
                    assert_eq!(value, "10 + 20");
                }
                _ => panic!("expected Assign"),
            }
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_expression() {
    let input = r#"
box test {
    run {
        serial_write_byte(0x41)
    }
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { body, .. } => {
            assert!(matches!(&body[0], Stmt::Expr(e) if e == "serial_write_byte(0x41)"));
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_multiple_boxes_and_flows() {
    let input = r#"
box serial {
    COM1 = 0x3F8
}

box vga {
    BASE = 0xB8000
}

flow default {
    serial
    vga
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    assert_eq!(prog.boxes.len(), 2);
    assert_eq!(prog.flows.len(), 1);
    assert_eq!(prog.boxes[0].name, "serial");
    assert_eq!(prog.boxes[1].name, "vga");
}

#[test]
fn test_parse_comments() {
    let input = r#"
// This is a comment
box serial {
    /* block comment */
    COM1 = 0x3F8 // inline comment
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    assert_eq!(prog.boxes.len(), 1);
    assert_eq!(prog.boxes[0].items.len(), 1);
}

#[test]
fn test_parse_type_inference() {
    let input = r#"
box test {
    write(s) => default
    read(buf) => default
    get(x) => default
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { params, .. } => {
            assert_eq!(params[0].1, "&str"); // s -> &str
        }
        _ => panic!("expected Fn"),
    }
    match &prog.boxes[0].items[1] {
        Item::Fn { params, .. } => {
            assert_eq!(params[0].1, "&[u32]"); // buf -> &[u32]
        }
        _ => panic!("expected Fn"),
    }
    match &prog.boxes[0].items[2] {
        Item::Fn { params, .. } => {
            assert_eq!(params[0].1, "u32"); // x -> u32 (default)
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_explicit_type() {
    let input = r#"
box test {
    write(x: u8) => default
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { params, .. } => {
            assert_eq!(params[0].1, "u8");
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_error_unterminated_box() {
    let input = r#"
box serial {
    COM1 = 0x3F8
"#;
    let mut p = Parser::new(input);
    let result = p.parse();
    assert!(result.is_err());
}

#[test]
fn test_parse_error_expected_keyword() {
    let input = r#"
invalid_keyword test {
}
"#;
    let mut p = Parser::new(input);
    let result = p.parse();
    assert!(result.is_err());
}

#[test]
fn test_parse_nested_blocks() {
    let input = r#"
box test {
    run {
        if x > 0 => {
            if y > 0 => {
                z = 1
            }
        }
    }
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { body, .. } => {
            match &body[0] {
                Stmt::If { then, .. } => {
                    assert_eq!(then.len(), 1);
                    assert!(matches!(&then[0], Stmt::If { .. }));
                }
                _ => panic!("expected If"),
            }
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_function_with_params_and_body() {
    let input = r#"
box test {
    add(a: u32, b: u32) {
        result = a + b
    }
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    match &prog.boxes[0].items[0] {
        Item::Fn { name, params, mode, body } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], ("a".to_string(), "u32".to_string()));
            assert_eq!(params[1], ("b".to_string(), "u32".to_string()));
            assert!(matches!(mode, FnMode::Block));
            assert_eq!(body.len(), 1);
        }
        _ => panic!("expected Fn"),
    }
}

#[test]
fn test_parse_program_display() {
    let input = r#"
box serial {
    COM1 = 0x3F8
    config => default
}
"#;
    let mut p = Parser::new(input);
    let prog = p.parse().unwrap();
    let display = format!("{}", prog);
    assert!(display.contains("box serial"));
    assert!(display.contains("COM1 = 0x3F8"));
    // The Display trait writes " => default" with a trailing newline
    assert!(display.contains("=> default"));
}
