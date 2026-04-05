use crate::types::Options;

/// Get the bullet character for unordered lists.
///
/// Validates and returns the configured bullet character.
pub fn check_bullet(options: &Options) -> char {
    let bullet = options.bullet;
    if bullet != '*' && bullet != '+' && bullet != '-' {
        panic!(
            "Cannot serialize items with `{}` for `options.bullet`, expected `*`, `+`, or `-`",
            bullet
        );
    }
    bullet
}

/// Get the alternative bullet character for unordered lists.
///
/// Used when the primary bullet can't be used (e.g., two adjacent lists).
pub fn check_bullet_other(options: &Options) -> char {
    let bullet = check_bullet(options);
    let bullet_other = options.bullet_other.unwrap_or(if bullet == '*' {
        '-'
    } else {
        '*'
    });

    if bullet_other != '*' && bullet_other != '+' && bullet_other != '-' {
        panic!(
            "Cannot serialize items with `{}` for `options.bulletOther`, expected `*`, `+`, or `-`",
            bullet_other
        );
    }

    if bullet_other == bullet {
        panic!(
            "Expected `options.bullet` (`{}`) and `options.bulletOther` (`{}`) to be different",
            bullet, bullet_other
        );
    }

    bullet_other
}

/// Get the bullet character for ordered lists.
pub fn check_bullet_ordered(options: &Options) -> char {
    let bullet_ordered = options.bullet_ordered;
    if bullet_ordered != '.' && bullet_ordered != ')' {
        panic!(
            "Cannot serialize items with `{}` for `options.bulletOrdered`, expected `.` or `)`",
            bullet_ordered
        );
    }
    bullet_ordered
}

/// Get the emphasis marker character.
pub fn check_emphasis(options: &Options) -> char {
    let emphasis = options.emphasis;
    if emphasis != '*' && emphasis != '_' {
        panic!(
            "Cannot serialize emphasis with `{}` for `options.emphasis`, expected `*` or `_`",
            emphasis
        );
    }
    emphasis
}

/// Get the fence marker character.
pub fn check_fence(options: &Options) -> char {
    let fence = options.fence;
    if fence != '`' && fence != '~' {
        panic!(
            "Cannot serialize code with `{}` for `options.fence`, expected `` ` `` or `~`",
            fence
        );
    }
    fence
}

/// Get the quote marker character.
pub fn check_quote(options: &Options) -> char {
    let quote = options.quote;
    if quote != '"' && quote != '\'' {
        panic!(
            "Cannot serialize title with `{}` for `options.quote`, expected `\"` or `'`",
            quote
        );
    }
    quote
}

/// Get the thematic break rule marker.
pub fn check_rule(options: &Options) -> char {
    let rule = options.rule;
    if rule != '*' && rule != '-' && rule != '_' {
        panic!(
            "Cannot serialize rules with `{}` for `options.rule`, expected `*`, `-`, or `_`",
            rule
        );
    }
    rule
}

/// Get the thematic break repetition count.
///
/// Returns the configured repetition, minimum 3.
pub fn check_rule_repetition(options: &Options) -> usize {
    let repetition = options.rule_repetition;
    if repetition < 3 {
        panic!(
            "Cannot serialize rules with repetition `{}` for `options.ruleRepetition`, expected a number of 3 or more",
            repetition
        );
    }
    repetition
}

/// Get the strong marker character.
pub fn check_strong(options: &Options) -> char {
    let strong = options.strong;
    if strong != '*' && strong != '_' {
        panic!(
            "Cannot serialize strong with `{}` for `options.strong`, expected `*` or `_`",
            strong
        );
    }
    strong
}
