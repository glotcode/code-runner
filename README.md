# code-runner


## Overview
code-runner is a language-agnostic command line application that reads code as a
JSON payload from stdin, runs the supplied build and run commands, and writes
the result as json to stdout.
This is used by [glot-languages](https://github.com/glotcode/glot-languages) to run code on [glot.io](https://glot.io)
See the [overview](https://github.com/glotcode/glot) on how everything is connected.


## Input (stdin)
The input is required to be a JSON object containing the properties `runInstructions`,
`files` and `stdin`. `files` must be an array with at least one object containing the
properties `name` and `content`. `name` is the name of the file and can include
forward slashes to create the file in a subdirectory. File names must be unique,
non-empty relative paths and cannot contain parent-directory components. Empty
file content is allowed. All files are written into the same base directory
under the OS's temp dir.


The caller is responsible for choosing non-empty commands appropriate for its runtime
image.


## Output (stdout)
The output is a JSON object containing the properties `stdout`, `stderr`, `error` and
`duration`. `stdout` and `stderr` contain the captured process output, `error` is
populated for compiler/interpreter failures, and `duration` is measured in nanoseconds.

## Examples

### Simple example
##### Input
```javascript
{
  "runInstructions": {
    "buildCommands": [],
    "runCommand": "python main.py"
  },
  "files": [
    {
      "name": "main.py",
      "content": "print(42)"
    }
  ],
  "stdin": null
}
```

##### Output
```javascript
{
  "stdout": "42\n",
  "stderr": "",
  "error": "",
  "duration": 123456
}
```

### Read from stdin
##### Input
```javascript
{
  "runInstructions": {
    "buildCommands": [],
    "runCommand": "python main.py"
  },
  "files": [
    {
      "name": "main.py",
      "content": "print(input('Number from stdin: '))"
    }
  ],
  "stdin": "42"
}
```

##### Output
```javascript
{
  "stdout": "Number from stdin: 42\n",
  "stderr": "",
  "error": "",
  "duration": 123456
}
```
