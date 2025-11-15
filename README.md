[![Crates.io Version](https://img.shields.io/crates/v/abyss-lang)](https://crates.io/crates/abyss-lang)
[![Build](https://github.com/liebe-magi/abyss/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/liebe-magi/abyss/actions/workflows/build.yml)

# **AbySS: Advanced-scripting by Symbolic Syntax**

![logo](/img/logo_256.png)

AbySS (Advanced-scripting by Symbolic Syntax) is a programming language designed to combine the thrill of casting spells with the power of advanced scripting. AbySS aims to provide an intuitive and symbolically rich syntax that allows developers to interact with their code as if they were performing magic. Whether you're scripting a simple operation or crafting complex systems, AbySS offers a unique and immersive experience.

## **Key Features**

- **Symbolic Syntax**: AbySS emphasizes a symbolically intuitive syntax, making the code easy to read and write, while retaining powerful functionality.
- **Spellcasting-inspired Programming**: The language's design mimics the experience of casting spells, with reserved keywords that evoke a magical theme.
- **Interactive Spellcasting**: AbySS supports interactive scripting through an interpreter, allowing real-time execution and feedback.
- **Structured Sorcery**: AbySS encourages structured programming, combining the flexibility of scripting with the rigor of structured code.
- **Arcane Collections**: Native `scroll`, `lexicon`, and `materia` types provide first-class list/dictionary semantics plus built-in rituals for inspecting and mutating data.
- **Rust-native Parser & Diagnostics**: The compiler core uses `chumsky` combinators paired with `ariadne` to provide resilient parsing and AbySS-themed error messages.
- **VSCode Extension**: Syntax highlighting, code completion, and snippets are available through the [AbySS Codex Familiar](https://github.com/liebe-magi/abyss-codex-familiar) VSCode extension.

## **Table of Contents**
- [Installation](#installation)
- [Getting Started](#getting-started)
- [Language Syntax](#language-syntax)
  - [Basic Syntax](#basic-syntax)
  - [Types](#types)
  - [Type Casting](#type-casting)
  - [Variable Declaration](#variable-declaration)
  - [Conditionals](#conditionals)
  - [Loops](#loops)
  - [Functions](#functions)
  - [Collections](#collections)
  - [Input/Output](#inputoutput)
- [VSCode Extension](#vscode-extension)
- [Roadmap](#roadmap)
- [License](#license)

## **Installation**

You can install AbySS from [crates.io](https://crates.io/crates/abyss-lang) using Cargo:

```bash
cargo install abyss-lang
```

Alternatively, you can install AbySS by cloning the repository and building it locally. `cargo-llvm-cov` is supported for test coverage analysis.

```bash
git clone https://github.com/your-repository/abyss.git
cd abyss
cargo install --path .
```

For test coverage with `cargo-llvm-cov`, install the tool as follows:

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

## **Getting Started**

To start using AbySS, you can either enter the interactive interpreter mode or run `.aby` script files.

### **Running the Interpreter**

You can start the AbySS interpreter with the following command:

```bash
abyss cast
```

### **Running Scripts**

To run a `.aby` script file, use the following command:

```bash
abyss invoke <script.aby>
```

### **Formatting Code**

AbySS provides a built-in code formatter that helps maintain consistent code style across your scripts. To format your `.aby` scripts, use the following command:

```bash
abyss align <script.aby>
```

This command automatically formats your code according to the language's style guidelines.

## **Language Syntax**

### **Basic Syntax**

AbySS uses a symbolic and intuitive syntax inspired by magical themes. Below are some of the core elements of the language.

- **Comments**: Comments in AbySS are marked with `//` for single-line comments and `/* */` for multi-line comments.

```abyss
// This is a single-line comment
/*
  This is a multi-line comment
*/
```

### **Types**

AbySS supports the following primitive types:
- **arcana**: Represents integers (e.g., `42`, `-3`).
- **aether**: Represents floating-point numbers (e.g., `3.14`, `-1.0`).
- **rune**: Represents strings (e.g., `"Hello, World"`).
- **omen**: Represents boolean values, with `boon` for `true` and `hex` for `false`.
- **abyss**: Represents the `void` type, indicating no value.
- **scroll**: Ordered collections that can mix any element types.
- **lexicon**: Rune-keyed dictionaries for structured data.
- **materia**: A dynamically typed slot that can store any runtime value.

```abyss
forge x: arcana = 10;
forge pi: aether = 3.14;
forge message: rune = "Hello, AbySS";
forge is_active: omen = boon;
forge spellbook: scroll = [1, boon, "sigil"];
forge ledger: lexicon = {"sun": 1, "moon": 2};
forge essence: materia = 99;
```

- `arcana`: Derived from the Latin word for "secret" or "mystery," and also referencing the structured numbering of tarot cards, arcana represents integer values, symbolizing the hidden foundations of calculations in programming.
- `aether`: Inspired by the classical element of ether, believed to fill the universe beyond the terrestrial sphere, aether represents floating-point numbers, capturing the idea of fluid, continuous quantities.
- `rune`: Borrowed from ancient writing systems used in magic, rune represents strings, suggesting the idea that words and symbols hold power in programming as they do in mystical inscriptions.
- `omen`: Drawing from the concept of a prophetic sign, omen represents boolean values. The keywords `boon` and `hex` are used for `true` and `false`, respectively — `boon` originates from Old English, meaning a blessing or benefit, while `hex` comes from Germanic folklore, signifying a curse or spell, reinforcing the mystical theme of the language.
- `abyss`: Symbolizing infinite nothingness, abyss represents the void type, indicating no value is returned, and is also the name of the language, reflecting its philosophy of exploring the depths of symbolic scripting.
- `scroll`: Modeled after arcane parchment rolls, scroll values are indexed by arcana and preserve insertion order, making them ideal for spell queues or mixed data payloads.
- `lexicon`: Inspired by grimoires and dictionaries, lexicon values map rune keys to arbitrary entries and allow quick lookups by string-like identifiers.
- `materia`: Named after alchemical prime matter, materia variables willingly hold any type; they are perfect for staging data before you know its final shape or when writing polymorphic helpers.

### **Type Casting**

In AbySS, type casting is achieved using the `trans` keyword.
This allows the conversion of values from one type to another.

`trans`: Short for "transformation," `trans` enables the conversion of one type into another, reflecting the idea of magical transformations in programming.

```abyss
forge x: arcana = trans(3.14 as arcana);
```

### **Variable Declaration**

Variables are declared using the `forge` keyword.
You must explicitly specify the variable type.

```abyss
forge x: arcana = 42;
forge greeting: rune = "Welcome to AbySS!";
```

To declare mutable variables, use the `morph` keyword with `forge`.

```abyss
forge morph counter: arcana = 10;
counter += 5;
```

- `forge`: Derived from the concept of a blacksmith forging items, this keyword represents the creation and declaration of new variables, symbolizing the act of crafting something new.
- `morph`: Inspired by transformation, morph is used to indicate mutable variables that can change their form or value over time.

### **Conditionals**

In AbySS, you can use the `oracle` construct to handle conditionals. One approach is to define conditions directly within each pattern, allowing for flexible and readable branching logic.
This method lets you skip a separate conditional statement and write all conditions within the branches themselves.

- `oracle`: Reflecting the role of an oracle in ancient mythology, this keyword is used for conditionals, where decisions and predictions are made based on input.

#### **Pattern-based Oracle**

```abyss
forge x: arcana = -5;
oracle {
    (x > 0) => unveil("x is positive");
    (x < 0) => unveil("x is negative");
    _ => unveil("x is zero");
};
```

In this example, the conditions are written directly within the patterns.
The oracle evaluates each condition in sequence and executes the matching branch.
If no specific conditions are met, the default pattern `_` is executed.

#### **Boolean-based Oracle**

Another way to use `oracle` is by explicitly writing a condition as part of the `oracle` expression, which can be evaluated as an `omen` (boolean) value.

```abyss
oracle (x > 0) {
    (boon) => unveil("x is positive");
    (hex) => unveil("x is non-positive");
};
```

Here, the condition `x > 0` is evaluated as `boon` (true) or `hex` (false), and the appropriate branch executes based on this result.

#### **Value-based Oracle**

You can also use expressions or assignments within `oracle` for more complex evaluations.

```abyss
forge y: arcana = 42;
oracle (z = y * 2) {
    (z > 50) => unveil("z is greater than 50");
    (z == 50) => unveil("z equals 50");
    _ => unveil("z is less than 50");
};
```

In this case, `z` is assigned the result of `y * 2`, and the appropriate branch is chosen based on the computed value of `z`.

#### **Multiple Condition-based Oracle**

You can also handle multiple conditions within a single oracle construct, allowing for complex branching based on multiple variables.

```abyss
forge a: arcana = 3;
forge b: arcana = 2;

oracle (a, b) {
    (1, 2) => unveil("a is 1 and b is 2");
    (3, 2) => unveil("a is 3 and b is 2");
    _ => unveil("Other combination");
};
```

In this example, oracle evaluates multiple conditions (a and b) together and executes the corresponding branch based on their values.
If no specific conditions are met, the default pattern _ is used.

This flexibility makes `oracle` a powerful tool for creating readable and intuitive branching logic in AbySS.

### **Loops**

In AbySS, loops are managed using the `orbit` keyword.
This construct allows for both simple and complex loop structures, with additional control provided by the `resume` and `eject` keywords.
Below, we explore a range of use cases for loops in AbySS, from basic to advanced scenarios.

- `orbit`: Inspired by the celestial motion of planets, orbit represents loops, where the code revolves around a central task repeatedly until a condition changes.
- `resume`: Reflecting the continuation of interrupted tasks, resume allows the loop to skip the current iteration and move on to the next.
- `eject`: Analogous to an emergency exit or escape mechanism, eject breaks out of loops, terminating the iteration prematurely.

#### **Simple Loop**

The most basic form of a loop in AbySS iterates over a range of values.
In this example, the loop iterates from 0 to 5 (exclusive of 5).

```abyss
orbit (i = 0..5) {
    unveil(i);
}
```

This loop prints the numbers from 0 to 4 using the `unveil` function.
The `..` operator defines a half-open range, which excludes the upper bound.

#### **Closed Range Loop**

You can create a closed range loop using the `..=` operator, which includes the upper bound in the iteration.

```abyss
orbit (i = 0..=10) {
    unveil(i);
}
```

In this loop, the numbers from 0 to 10 (inclusive) are printed.

#### **Infinite Loop**

By omitting the loop parameters, you can create an infinite loop that runs until a certain condition is met.
The loop can be terminated using the `eject` keyword, which breaks the loop when the condition is satisfied.

```abyss
forge morph i: arcana = 0;
orbit {
    oracle (i == 100) {
        (boon) => eject; // Break the loop when i equals 100
    };
    i += 1;
};
unveil(i); // Prints 100 after loop termination
```
This example increments `i` in each iteration until it reaches 100, at which point the loop exits.

#### **Loop with Multiple Parameters**

AbySS allows you to define loops with more than one parameter, making it easier to iterate over multiple ranges simultaneously.
Here's an example where two loop variables, `i` and `j`, are defined:

```abyss
orbit (i = 0..3, j = 0..3) {
    unveil(i, " ", j);
}
```

In this loop, `i` and `j` both iterate over the range from 0 to 2, and each pair of values is printed.
This pattern is useful when you need to perform operations that depend on two or more varying values simultaneously.

#### **Resume Keyword**

The `resume` keyword allows you to skip the current iteration of a loop and move on to the next iteration.
This can be used to control the flow of nested loops.

```abyss
orbit (i = 0..3) {
    orbit (j = 0..3) {
        oracle (i == j) {
            (boon) => resume j; // Skip the iteration when i equals j
        };
        unveil(i, " ", j);
    }
}
```

In this example, the inner loop skips the iteration whenever `i` is equal to `j`.

#### **Eject Keyword**

The `eject` keyword breaks the loop entirely.
In nested loops, you can specify which loop to break by passing the loop variable as an argument.

```abyss
orbit (i = 0..3) {
    orbit (j = 0..3) {
        unveil(i, " ", j);
        oracle (i == 2) {
            (boon) => eject i; // Break the outer loop when i equals 2
        };
    }
}
```

Here, the outer loop breaks entirely when `i` equals 2, effectively terminating both the inner and outer loops.

These examples illustrate the flexibility of the `orbit` construct in AbySS, which allows for sophisticated looping patterns with easy-to-read syntax.
The ability to control flow with `resume` and `eject` further enhances the language's expressiveness in handling loops.

### **Functions**

In AbySS, functions are defined using the `engrave` keyword.
Functions allow you to encapsulate reusable code blocks and return values.
You can define functions with parameters, specify return types, and call these functions within your scripts.

- `engrave`: Symbolizing the act of carving a magical circle, engrave is used for function definitions, representing the meticulous process of inscribing reusable spells into the code.

#### **Function Definition**

Functions are defined using the `engrave` keyword, followed by the function name, parameters, optional return type, and the function body enclosed in `{}`.

```abyss
engrave add(a: arcana, b: arcana) -> arcana {
    reveal a + b;
};
```

In this example, the `add` function takes two parameters `a` and `b` of type `arcana` (integer), and returns their sum as an `arcana`.

#### **Function Calls**

You can call a function by using its name followed by parentheses, and pass arguments matching the function's parameters.

```abyss
forge result: arcana = add(3, 5);
unveil(result); // Outputs: 8
```

This example calls the `add` function with the arguments `3` and `5`, stores the result in the `result` variable, and prints it using `unveil`.

#### **Nested Function Calls**

AbySS allows you to call functions within other function calls, enabling complex operations to be broken down into simpler components.

```abyss
engrave add_three_numbers(x: arcana, y: arcana, z: arcana) -> arcana {
    reveal add(add(x, y), z);
};

forge result: arcana = add_three_numbers(1, 2, 3);
unveil(result); // Outputs: 6
```

In this example, the `add_three_numbers` function calls the `add` function twice to sum three numbers.

#### **Return Values**

Functions in AbySS use the `reveal` keyword to return values.
If a function does not explicitly return a value, it implicitly returns `abyss`.

```abyss
engrave greet() -> rune {
    reveal "Hello, AbySS!";
};

unveil(greet()); // Outputs: Hello, AbySS!
```

This function `greet` returns a `rune` (string) and prints it using `unveil`.

#### **Recursive Functions**

AbySS supports recursive function calls, allowing functions to call themselves.

```abyss
engrave factorial(n: arcana) -> arcana {
    oracle (n <= 1) {
        (boon) => reveal 1;
    };
    reveal n * factorial(n - 1);
};

forge result: arcana = factorial(5);
unveil(result); // Outputs: 120
```

This recursive function calculates the factorial of a number.

### **Collections**

Scrolls and lexicons bring first-class collection support to AbySS. Scrolls behave like ordered lists, while lexicons act as rune-keyed dictionaries. Literals use familiar JSON-like syntax, and you can index into either structure with bracket notation.

```abyss
forge spellbook: scroll = ["ignite", 42, boon];
forge grimoire: lexicon = {
    "sun": "radiant",
    "moon": "serene",
};

unveil(spellbook[0]);        // "ignite"
unveil(grimoire["moon"]);   // "serene"

forge morph satchel: scroll = [1, 2];
satchel[1] = 3;               // morph required for mutation
```

Mutation requires the `morph` keyword, ensuring immutable data stays safe by default. Nested indexing is also supported, so `lexicon_scroll["sigils"][0]` works as expected.

#### Collection rituals

The standard library exposes helper spells for manipulating collections:

- `measure(collection)` – returns the length of a scroll or lexicon as `arcana`.
- `inscribe(scroll, value)` – appends to a morph scroll.
- `retract(scroll)` – pops and returns the last entry of a morph scroll.
- `expunge(lexicon, "key")` – removes a rune key from a morph lexicon.
- `contents(lexicon)` – reveals a scroll of the stored rune keys.

```abyss
forge morph pack: scroll = ["ember"];
inscribe(pack, "frost");
unveil(measure(pack)); // 2
unveil(retract(pack)); // "frost"

forge morph ledger: lexicon = {"alpha": 1, "beta": 2};
expunge(ledger, "alpha");
unveil(contents(ledger)); // ["beta"]
```

These rituals operate on runtime `EvalResult`s directly, so you can pass nested scrolls or lexicons without additional boilerplate.

### **Input/Output**

For output, AbySS uses the `unveil` function to print values to the console.
You can pass multiple variables or expressions as arguments, separated by commas.
This allows you to concatenate different elements in the output.

- `unveil`: Chosen for its metaphorical meaning of revealing something hidden, unveil is used for output operations, making the internal state of the program visible to the user.

```abyss
unveil("Hello, AbySS");
unveil("x + 42 = ", x + 42);
```

In the example above, the second `unveil` statement prints both the string `"x + 42 = "` and the result of the expression `x + 42` on the same line.

For input, AbySS provides the `summon` function to read user input during script execution.
The `summon` function takes a prompt message, always returns a `rune`, and callers can use `trans` to convert it into other types when needed.

- `summon`: Represents the act of calling forth something from the user, summon is used for standard input operations. Provide a prompt rune, and use `trans` to coerce the response into `arcana` or `aether` when numerical input is required.

```abyss
forge name: rune = summon("Input your name: ");
forge age: arcana = trans(summon("Input your age: ") as arcana);
unveil("Hello, ", name, "! You are ", age, " years old.");
```

In this example, the user is prompted to enter their name and age.
The string inputs are stored directly for `name`, while `age` is converted from the returned rune into an `arcana` using `trans` before being printed with `unveil`.

## **VSCode Extension**

The [AbySS Codex Familiar](https://github.com/liebe-magi/abyss-codex-familiar) VSCode extension provides additional support for AbySS development, including:
- Syntax highlighting for keywords, types, constants, and operators.
- Code snippets for common structures like `forge`, `unveil`, and `oracle`.
- Auto-completion for key AbySS constructs.

To install the extension, search for "[AbySS Codex Familiar](https://marketplace.visualstudio.com/items?itemName=liebe-magi.abyss-codex-familiar)" in the Visual Studio Code Extensions Marketplace, or download it from the [GitHub repository](https://github.com/liebe-magi/abyss-codex-familiar).

### **Roadmap**

- **Collection Types**: ✅ Scroll, lexicon, and materia shipped with literal syntax, indexing, and stdlib rituals.
- **Struct Implementation**: Enable the definition and use of custom data structures (TBD).
- **Generics Introduction**: Introduce generics to allow functions and data structures to be more flexible and reusable with different types (TBD).
- **Module System**: Introduce the ability to import functions and variables from other files (TBD).
- **Error Handling**: Implement robust error handling (TBD).
- **File I/O**: Introduce input functionality and file handling (TBD).
- **Standard Library**: Develop a standard library with common functions and utilities (TBD).
- **Interpreter Enhancements**: Improve the interactive interpreter with better real-time feedback, debugging capabilities, and performance optimizations (TBD).

## **License**

AbySS is open-source software licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
