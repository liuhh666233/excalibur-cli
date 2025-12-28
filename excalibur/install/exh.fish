# Excalibur - Fish Shell Integration
#
# Function name: exh (excalibur history)

function exh --description "Interactive command history browser (Excalibur)"
    # Directly call the excalibur binary
    set -l selected_cmd (command excalibur 2>/dev/null)

    # Get the exit status
    set -l status_code $status

    # Clear any residual output from TUI
    commandline -f repaint

    # If user selected a command (exit code 0 or 10) and output is not empty
    if test -n "$selected_cmd"
        if test $status_code -eq 0
            # Exit code 0: Insert command into command line (user can edit)
            commandline -r -- $selected_cmd
            commandline -f repaint
        else if test $status_code -eq 10
            # Exit code 10: Insert and execute immediately
            commandline -r -- $selected_cmd
            commandline -f repaint
            commandline -f execute
        else
            # Other exit codes: just repaint
            commandline -f repaint
        end
    else
        # User cancelled or error occurred, just repaint
        commandline -f repaint
    end
end

# Bind to Ctrl+R (overwrites default Fish history search)
bind \cr exh

# Optional: Bind to Ctrl+H as well
# bind \ch exh
