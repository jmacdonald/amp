# Runs amp with suppressed debug output
run:
  cargo run 2> /dev/null

stderr_file := "stderr"
# Runs amp with a temporary vertical split pane tailing debug output
debug:
  #!/bin/sh
  touch {{stderr_file}}
  tail_pane_id=$(tmux split-window -v -d -P -F '#{pane_id}' "tail -f {{stderr_file}}")
  RUST_LOG=debug cargo run 2> {{stderr_file}}
  tmux kill-pane -t "$tail_pane_id"
  rm {{stderr_file}}

check:
  cargo check

test:
  cargo test

# Serve docs on port 8000
[group('docs')]
docs:
  docker run --rm -it -p 8000:8000 -v ${PWD}/documentation:/docs wastedintel/zensical

# Build docs to documentation/site
[group('docs')]
build_docs:
  docker run --rm -it -v ${PWD}/documentation:/docs wastedintel/zensical zensical build

# Build Zensical Docker image
[group('docs')]
build_docs_image:
  docker build documentation/ -t wastedintel/zensical
