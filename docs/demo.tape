# nm-remover demo
# Prerequisites: ./docs/setup-demo.sh && cargo build --release
# Run: vhs docs/demo.tape

Output docs/demo.gif

Set Shell zsh
Set FontSize 18
Set Width 1200
Set Height 700
Set Padding 10
Set Theme "Catppuccin Mocha"
Set TypingSpeed 50ms
Set PlaybackSpeed 1

# Launch
Type "./target/release/nm-remover"
Sleep 500ms
Enter
Sleep 1.5s

# Enter path on welcome screen
Type "/tmp/nm-remover-demo"
Sleep 500ms
Enter
Sleep 2s

# Browse results
Sleep 1s
Down
Sleep 400ms
Down
Sleep 400ms
Down
Sleep 400ms

# Select items with Space
Up
Sleep 300ms
Space
Sleep 400ms
Up
Sleep 300ms
Space
Sleep 800ms

# Select all
Type "a"
Sleep 1s

# Deselect all
Type "n"
Sleep 1s

# Show help popup
Type "?"
Sleep 2.5s

# Close help
Space
Sleep 800ms

# Select top 2 for delete demo
Space
Sleep 300ms
Down
Sleep 300ms
Space
Sleep 800ms

# Trigger delete confirmation
Type "d"
Sleep 2.5s

# Cancel
Type "n"
Sleep 1s

# Quit
Type "q"
Sleep 1s
