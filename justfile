run:
  cargo run 2> /dev/null

# Runs amp with a temporary vertical split pane tailing debug output
stderr_file := "stderr"
debug:
  #!/bin/sh
  touch {{stderr_file}}
  tail_pane_id=$(tmux split-window -v -d -P -F '#{pane_id}' "tail -f {{stderr_file}}")
  cargo run 2> {{stderr_file}}
  tmux kill-pane -t "$tail_pane_id"
  rm {{stderr_file}}

check:
  cargo check

test:
  cargo test
