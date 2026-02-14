# Excalibur - Fish Shell Integration
#
# Function name: excc (excalibur claude code settings)

function excc --description "Switch Claude Code settings profiles (Excalibur)"
    command excalibur s 2>/dev/null
    commandline -f repaint
end
