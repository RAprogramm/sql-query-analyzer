// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

use sql_query_analyzer::rules::{AnalysisReport, RuleCategory, RuleInfo, Severity, Violation};

#[test]
fn test_severity_display_info() {
    let s = Severity::Info;
    assert_eq!(format!("{}", s), "INFO");
}

#[test]
fn test_severity_display_warning() {
    let s = Severity::Warning;
    assert_eq!(format!("{}", s), "WARN");
}

#[test]
fn test_severity_display_error() {
    let s = Severity::Error;
    assert_eq!(format!("{}", s), "ERROR");
}

#[test]
fn test_severity_ordering() {
    assert!(Severity::Info < Severity::Warning);
    assert!(Severity::Warning < Severity::Error);
    assert!(Severity::Info < Severity::Error);
}

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Info, Severity::Info);
    assert_ne!(Severity::Info, Severity::Warning);
}

#[test]
fn test_severity_clone() {
    let s = Severity::Error;
    let cloned = s.clone();
    assert_eq!(s, cloned);
}

#[test]
fn test_severity_debug() {
    let s = Severity::Warning;
    let debug = format!("{:?}", s);
    assert!(debug.contains("Warning"));
}

#[test]
fn test_rule_category_display_performance() {
    let c = RuleCategory::Performance;
    assert_eq!(format!("{}", c), "Performance");
}

#[test]
fn test_rule_category_display_style() {
    let c = RuleCategory::Style;
    assert_eq!(format!("{}", c), "Style");
}

#[test]
fn test_rule_category_display_security() {
    let c = RuleCategory::Security;
    assert_eq!(format!("{}", c), "Security");
}

#[test]
fn test_rule_category_equality() {
    assert_eq!(RuleCategory::Performance, RuleCategory::Performance);
    assert_ne!(RuleCategory::Performance, RuleCategory::Style);
}

#[test]
fn test_rule_category_clone() {
    let c = RuleCategory::Security;
    let cloned = c.clone();
    assert_eq!(c, cloned);
}

#[test]
fn test_rule_category_debug() {
    let c = RuleCategory::Style;
    let debug = format!("{:?}", c);
    assert!(debug.contains("Style"));
}

#[test]
fn test_violation_creation() {
    let v = Violation {
        rule_id:     "TEST001",
        rule_name:   "Test Rule",
        message:     "Test message".to_string(),
        severity:    Severity::Warning,
        category:    RuleCategory::Performance,
        suggestion:  Some("Fix it".to_string()),
        query_index: 0
    };
    assert_eq!(v.rule_id, "TEST001");
    assert_eq!(v.rule_name, "Test Rule");
    assert_eq!(v.message, "Test message");
}

#[test]
fn test_violation_without_suggestion() {
    let v = Violation {
        rule_id:     "TEST002",
        rule_name:   "Test Rule 2",
        message:     "Test message".to_string(),
        severity:    Severity::Info,
        category:    RuleCategory::Style,
        suggestion:  None,
        query_index: 1
    };
    assert!(v.suggestion.is_none());
}

#[test]
fn test_violation_clone() {
    let v = Violation {
        rule_id:     "TEST003",
        rule_name:   "Test Rule 3",
        message:     "Test message".to_string(),
        severity:    Severity::Error,
        category:    RuleCategory::Security,
        suggestion:  None,
        query_index: 2
    };
    let cloned = v.clone();
    assert_eq!(cloned.rule_id, v.rule_id);
    assert_eq!(cloned.message, v.message);
}

#[test]
fn test_violation_debug() {
    let v = Violation {
        rule_id:     "TEST004",
        rule_name:   "Test Rule 4",
        message:     "Test".to_string(),
        severity:    Severity::Warning,
        category:    RuleCategory::Performance,
        suggestion:  None,
        query_index: 0
    };
    let debug = format!("{:?}", v);
    assert!(debug.contains("TEST004"));
}

#[test]
fn test_rule_info_creation() {
    let info = RuleInfo {
        id:       "PERF001",
        name:     "Select Star",
        severity: Severity::Warning,
        category: RuleCategory::Performance
    };
    assert_eq!(info.id, "PERF001");
    assert_eq!(info.name, "Select Star");
}

#[test]
fn test_rule_info_clone() {
    let info = RuleInfo {
        id:       "SEC001",
        name:     "SQL Injection",
        severity: Severity::Error,
        category: RuleCategory::Security
    };
    let cloned = info.clone();
    assert_eq!(cloned.id, info.id);
    assert_eq!(cloned.name, info.name);
}

#[test]
fn test_rule_info_debug() {
    let info = RuleInfo {
        id:       "STYLE001",
        name:     "Style Rule",
        severity: Severity::Info,
        category: RuleCategory::Style
    };
    let debug = format!("{:?}", info);
    assert!(debug.contains("STYLE001"));
}

#[test]
fn test_analysis_report_new() {
    let report = AnalysisReport::new(5, 10);
    assert_eq!(report.queries_count, 5);
    assert_eq!(report.rules_count, 10);
    assert!(report.violations.is_empty());
}

#[test]
fn test_analysis_report_add_violation() {
    let mut report = AnalysisReport::new(1, 1);
    report.add_violation(Violation {
        rule_id:     "TEST",
        rule_name:   "Test",
        message:     "Test".to_string(),
        severity:    Severity::Warning,
        category:    RuleCategory::Performance,
        suggestion:  None,
        query_index: 0
    });
    assert_eq!(report.violations.len(), 1);
}

#[test]
fn test_analysis_report_counts() {
    let mut report = AnalysisReport::new(1, 1);

    report.add_violation(Violation {
        rule_id:     "E1",
        rule_name:   "Error",
        message:     "Error".to_string(),
        severity:    Severity::Error,
        category:    RuleCategory::Security,
        suggestion:  None,
        query_index: 0
    });

    report.add_violation(Violation {
        rule_id:     "W1",
        rule_name:   "Warning",
        message:     "Warning".to_string(),
        severity:    Severity::Warning,
        category:    RuleCategory::Performance,
        suggestion:  None,
        query_index: 0
    });

    report.add_violation(Violation {
        rule_id:     "I1",
        rule_name:   "Info",
        message:     "Info".to_string(),
        severity:    Severity::Info,
        category:    RuleCategory::Style,
        suggestion:  None,
        query_index: 0
    });

    assert_eq!(report.error_count(), 1);
    assert_eq!(report.warning_count(), 1);
    assert_eq!(report.info_count(), 1);
}

#[test]
fn test_analysis_report_clone() {
    let mut report = AnalysisReport::new(2, 3);
    report.add_violation(Violation {
        rule_id:     "T1",
        rule_name:   "Test",
        message:     "Test".to_string(),
        severity:    Severity::Warning,
        category:    RuleCategory::Performance,
        suggestion:  None,
        query_index: 0
    });
    let cloned = report.clone();
    assert_eq!(cloned.violations.len(), report.violations.len());
}

#[test]
fn test_analysis_report_debug() {
    let report = AnalysisReport::new(1, 1);
    let debug = format!("{:?}", report);
    assert!(debug.contains("AnalysisReport"));
}

#[test]
fn test_severity_serialize() {
    let s = Severity::Error;
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.contains("Error"));
}

#[test]
fn test_rule_category_serialize() {
    let c = RuleCategory::Security;
    let json = serde_json::to_string(&c).unwrap();
    assert!(json.contains("Security"));
}

#[test]
fn test_violation_serialize() {
    let v = Violation {
        rule_id:     "SER001",
        rule_name:   "Serialize Test",
        message:     "Serialization test".to_string(),
        severity:    Severity::Warning,
        category:    RuleCategory::Style,
        suggestion:  Some("Suggestion".to_string()),
        query_index: 0
    };
    let json = serde_json::to_string(&v).unwrap();
    assert!(json.contains("SER001"));
    assert!(json.contains("Suggestion"));
}

#[test]
fn test_analysis_report_serialize() {
    let report = AnalysisReport::new(5, 10);
    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("violations"));
    assert!(json.contains("queries_count"));
}
