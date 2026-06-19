use crate::status::format_tokens_compact;
use codex_app_server_protocol::ThreadGoal;
use codex_app_server_protocol::ThreadGoalStatus;

pub(crate) const GOAL_USAGE: &str = "Usage: /goal [<objective>|clear|edit|pause|resume]";

pub(crate) fn format_goal_elapsed_seconds(seconds: i64) -> String {
    // Month and year buckets are approximate because this formats elapsed duration.
    const DAYS_PER_WEEK: u64 = 7;
    const APPROX_DAYS_PER_MONTH: u64 = 30;
    const APPROX_DAYS_PER_YEAR: u64 = 365;

    let seconds = seconds.max(0) as u64;
    if seconds < 60 {
        return format!("{seconds}s");
    }

    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{minutes}m");
    }

    let hours = minutes / 60;
    let remaining_minutes = minutes % 60;
    let total_days = hours / 24;
    let remaining_hours = hours % 24;

    if total_days >= APPROX_DAYS_PER_YEAR {
        let mut remaining_days = total_days;
        let years = remaining_days / APPROX_DAYS_PER_YEAR;
        remaining_days %= APPROX_DAYS_PER_YEAR;
        let months = remaining_days / APPROX_DAYS_PER_MONTH;
        remaining_days %= APPROX_DAYS_PER_MONTH;
        let weeks = remaining_days / DAYS_PER_WEEK;
        remaining_days %= DAYS_PER_WEEK;
        return format!(
            "{years}y {months}mo {weeks}w {remaining_days}d {remaining_hours}h {remaining_minutes}m"
        );
    }

    if total_days >= APPROX_DAYS_PER_MONTH {
        let months = total_days / APPROX_DAYS_PER_MONTH;
        let remaining_days = total_days % APPROX_DAYS_PER_MONTH;
        let weeks = remaining_days / DAYS_PER_WEEK;
        let remaining_days = remaining_days % DAYS_PER_WEEK;
        return format!(
            "{months}mo {weeks}w {remaining_days}d {remaining_hours}h {remaining_minutes}m"
        );
    }

    if total_days >= DAYS_PER_WEEK {
        let weeks = total_days / DAYS_PER_WEEK;
        let remaining_days = total_days % DAYS_PER_WEEK;
        return format!("{weeks}w {remaining_days}d {remaining_hours}h {remaining_minutes}m");
    }

    if total_days > 0 {
        return format!("{total_days}d {remaining_hours}h {remaining_minutes}m");
    }

    if remaining_minutes == 0 {
        format!("{hours}h")
    } else {
        format!("{hours}h {remaining_minutes}m")
    }
}

pub(crate) fn goal_status_label(status: ThreadGoalStatus) -> &'static str {
    match status {
        ThreadGoalStatus::Active => "active",
        ThreadGoalStatus::Paused => "paused",
        ThreadGoalStatus::Blocked => "blocked",
        ThreadGoalStatus::UsageLimited => "usage limited",
        ThreadGoalStatus::BudgetLimited => "limited by budget",
        ThreadGoalStatus::Complete => "complete",
    }
}

pub(crate) fn goal_usage_summary(goal: &ThreadGoal) -> String {
    let mut parts = vec![format!("Objective: {}", goal.objective)];
    if goal.time_used_seconds > 0 {
        parts.push(format!(
            "Time: {}.",
            format_goal_elapsed_seconds(goal.time_used_seconds)
        ));
    }
    if let Some(token_budget) = goal.token_budget {
        parts.push(format!(
            "Tokens: {}/{}.",
            format_tokens_compact(goal.tokens_used),
            format_tokens_compact(token_budget)
        ));
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_app_server_protocol::ThreadGoal;
    use codex_app_server_protocol::ThreadGoalStatus;
    use pretty_assertions::assert_eq;

    #[test]
    fn format_goal_elapsed_seconds_is_compact() {
        assert_eq!(format_goal_elapsed_seconds(/*seconds*/ 0), "0s");
        assert_eq!(format_goal_elapsed_seconds(/*seconds*/ 59), "59s");
        assert_eq!(format_goal_elapsed_seconds(/*seconds*/ 60), "1m");
        assert_eq!(format_goal_elapsed_seconds(30 * 60), "30m");
        assert_eq!(format_goal_elapsed_seconds(90 * 60), "1h 30m");
        assert_eq!(format_goal_elapsed_seconds(2 * 60 * 60), "2h");
        let just_before_one_day = 24 * 60 * 60 - 1;
        assert_eq!(format_goal_elapsed_seconds(just_before_one_day), "23h 59m");

        let one_day = 24 * 60 * 60;
        assert_eq!(format_goal_elapsed_seconds(one_day), "1d 0h 0m");

        let almost_three_days = 2 * 24 * 60 * 60 + 23 * 60 * 60 + 42 * 60;
        assert_eq!(format_goal_elapsed_seconds(almost_three_days), "2d 23h 42m");

        let day = 24 * 60 * 60;
        for (seconds, expected) in [
            (7 * day - 1, "6d 23h 59m"),
            (7 * day, "1w 0d 0h 0m"),
            (7 * day + 2 * 60 * 60 + 5 * 60, "1w 0d 2h 5m"),
            (30 * day - 1, "4w 1d 23h 59m"),
            (30 * day, "1mo 0w 0d 0h 0m"),
            (38 * day, "1mo 1w 1d 0h 0m"),
            (32 * day + 23 * 60 * 60 + 42 * 60, "1mo 0w 2d 23h 42m"),
            (365 * day, "1y 0mo 0w 0d 0h 0m"),
            (400 * day, "1y 1mo 0w 5d 0h 0m"),
        ] {
            assert_eq!(format_goal_elapsed_seconds(seconds), expected, "{seconds}s");
        }
    }

    fn test_thread_goal(token_budget: Option<i64>, tokens_used: i64) -> ThreadGoal {
        ThreadGoal {
            thread_id: "thread-1".to_string(),
            objective: "Complete the task described in ../gameboy-long-running-prompt5.txt"
                .to_string(),
            status: ThreadGoalStatus::BudgetLimited,
            token_budget,
            tokens_used,
            time_used_seconds: 120,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn goal_usage_summary_formats_time_and_budgeted_tokens() {
        assert_eq!(
            goal_usage_summary(&test_thread_goal(
                /*token_budget*/ Some(50_000),
                /*tokens_used*/ 63_876,
            )),
            "Objective: Complete the task described in ../gameboy-long-running-prompt5.txt Time: 2m. Tokens: 63.9K/50K."
        );
    }
}
