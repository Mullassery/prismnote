#!/usr/bin/env fish
# Fish completion for PrismNote

complete -c prismnote -f -n "__fish_use_subcommand_from_list" -d "Enterprise data science notebook"

# Help and version
complete -c prismnote -s h -l help -d "Show help message"
complete -c prismnote -s v -l version -d "Show version information"

# Server options
complete -c prismnote -l port -d "Server port number" -x
complete -c prismnote -l host -d "Server host address" -x
complete -c prismnote -l data -d "Data directory path" -x -a "(__fish_complete_directories)"

# Configuration options
complete -c prismnote -l log-level -d "Logging level" -x -a "debug\tDebug\\ level info\tInfo\\ level warn\tWarning\\ level error\tError\\ level"
complete -c prismnote -l config -d "Configuration file" -x -a "(__fish_complete_path)"
complete -c prismnote -l plugins -d "Plugins directory" -x -a "(__fish_complete_directories)"

# Environment options
complete -c prismnote -l auth-provider -d "Authentication provider" -x -a "aad\tMicrosoft\\ AAD ldap\tLDAP saml\tSAML oauth2\tOAuth2 none\tNo\\ auth"
complete -c prismnote -l spark-master -d "Spark master URL" -x
complete -c prismnote -l spark-executor-memory -d "Spark executor memory" -x

# Examples
complete -c prismnote -l help -d "Examples:
  prismnote                      Start server on http://localhost:8000
  prismnote --port 3000          Use custom port
  prismnote --data /custom/dir   Use custom data directory
  prismnote --log-level debug    Enable debug logging"
