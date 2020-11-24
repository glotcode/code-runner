#!/bin/bash

JSON=$(cat <<-END
{
    "language": "python",
    "files": [
        {
            "name": "main.py",
            "content": "print(42)"
        }
    ]
}
END)

echo $JSON | cargo run
