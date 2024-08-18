# code-runner


## Overview
code-runner is a command line application that reads code as a
json payload from stdin – compiles and runs the code – and writes
the result as json to stdout.
This is used by [glot-languages](https://github.com/glotcode/glot-languages) to run code on [glot.io](https://glot.io)
See the [overview](https://github.com/glotcode/glot) on how everything is connected.


## Input (stdin)
The input is required to be a json object containing the properties `runInstructions`,
`files` and `stdin`. `files` must be an array with at least one object containing the
properties `name` and `content`. `name` is the name of the file and can include
forward slashes to create the file in a subdirectory relative to the base
directory. All files are written into the same base directory under the OS's
temp dir.


## Output (stdout)
The output is a json object containing the properties `stdout`, `stderr` and
`error`. `stdout` and `stderr` is captured from the output of the ran code.
`error` is popuplated if there is a compiler / interpreter error.

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
  "error": ""
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
  "error": ""
}
```