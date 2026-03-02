## Builder Permission Packet
- proposal_id: builder_prop_10106_20106_proposal_sig_1_PH1_FEEDBACK
- change_class: CLASS-A
- issue_summary: PH1.FEEDBACK -> READ_ONLY_CLARIFY_LOOP
- fix_target: PRUNE_CLARIFICATION_ORDERING
- recommendation: proposal_sig_1_PH1_FEEDBACK
- change_brief_path: .dev/builder_change_brief.md
- learning_report_id: learn_report_builder_prop_10106_20106_proposal_sig_1_PH1_FEEDBACK (signals=1)

## Code Permission Request (BCAST)
- permission_ref: perm_code_builder_prop_10106_20106_proposal_sig_1_PH1_FEEDBACK
- message: Should I proceed?
- simulation_step_1: BCAST_CREATE_DRAFT
- simulation_step_2: BCAST_DELIVER_COMMIT
- busy_followup: REMINDER_SCHEDULE_COMMIT (reminder_type=BCAST_MHP_FOLLOWUP)

## Launch Permission Request (BCAST)
- permission_ref: perm_launch_builder_prop_10106_20106_proposal_sig_1_PH1_FEEDBACK
- message: All tests passed. Can I launch?
- simulation_step_1: BCAST_CREATE_DRAFT
- simulation_step_2: BCAST_DELIVER_COMMIT
- busy_followup: REMINDER_SCHEDULE_COMMIT (reminder_type=BCAST_MHP_FOLLOWUP)

## Apply Decision Commands
- code approve: BCAST_ID=<code_bcast_id> DECISION_REF=perm_code_builder_prop_10106_20106_proposal_sig_1_PH1_FEEDBACK ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh code approve
- launch approve: BCAST_ID=<launch_bcast_id> DECISION_REF=perm_launch_builder_prop_10106_20106_proposal_sig_1_PH1_FEEDBACK ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh launch approve
- code pending busy: REMINDER_REF=<code_reminder_ref> ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh code pending
- launch pending busy: REMINDER_REF=<launch_reminder_ref> ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh launch pending

## Auto-Generated Decision Files
- decision_file_template: docs/fixtures/builder_permission_decision_template.env
- code_decision_file: .dev/builder_code_decision.env
- launch_decision_file: .dev/builder_launch_decision.env
- apply code file: DECISION_FILE=.dev/builder_code_decision.env ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh
- apply launch file: DECISION_FILE=.dev/builder_launch_decision.env ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh

## Hard Rules
- No Approval -> No Code
- No Launch Approval -> No Launch
- Reminder scheduling does not grant approval
