# Personal planner

## Usage

```shell
planner 0.1.0

USAGE:
    planner [FLAGS] [OPTIONS] -i <source>

FLAGS:
    -h, --help       Prints help information
    -p               Send output to stdout
    -V, --version    Prints version information
    -w               Keeps watching source API file changes

OPTIONS:
    -o <output-file>        Output file [default: ./plan.json]
    -i <source>             Input folder

```

The input folder **must contain** two files with the exact file names: 
`plan-header.toml` (in TOML format) and `plan.fplan` (syntax described below).

An example of the toml file [can be found here](src/plan-header.toml).

An example of the plan file [can be found here](src/plan.fplan).

## Syntax

All building blocks are per single line, there is no support for multi-line 
elements.

Comments is supported with lines starting with `//`.

Identation of blocks is optional. Lines will be trimmed.

```
feature: <description>

// the feature links, link title should not use ':'
docs:
	<link title>: <URL>
	
task: <task description>

effort: <service alias as defined in toml> <points>

by: <resource alias as defined in toml> <sprint text>

notes:
	<a line of notes>
	<another line>

// task links
links:
	<link title>: <URL>
	<link title>: <URL>
	
ticket: <some ticket number or reference>

dependencies:
	<team as defined in toml> <sprint reference> <description>
```