// Copyright (c) 2025 Faisal A.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the “Software”), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::alloc::{self, Layout};
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

// NOTE: Alternatively, the user can choose their buffer size.
const MEMORY_SIZE: usize = 128;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        eprintln!("\x1b[31merror\x1b[m: no filepath provided.");
        process::exit(1);
    }

    let mut file_contents = String::new();
    let _ = read_file(&args[1], &mut file_contents);
    let cleaned_file_contents = remove_comments(&file_contents);
    let chars = cleaned_file_contents.chars().collect::<Vec<char>>();

    let mut memory = Memory::new(MEMORY_SIZE);
    // The loop manager keeps a stack of the indices for the loop-starts. Makes
    // it very easy to jump back to the correct index when going back to the
    // beginning of a loop. :)
    let mut loop_mng = Vec::with_capacity(16);

    let mut i = 0;

    while i < chars.len() {
        let op_res = handle_operation(chars[i], &mut memory);

        if op_res.is_some() {
            match op_res.unwrap() {
                IdxOp::MoveIdx => {
                    i = find_matching_bracket(&chars, i);
                    loop_mng.pop();
                }
                IdxOp::SaveIdx => loop_mng.push(i),
                // WARNING: If there are an incomplete number of
                IdxOp::BackIdx => i = *loop_mng.last().unwrap(),
            }
        }

        i += 1;
    }

    debugging::print_memory_block(0, 10, &memory);
}

fn find_matching_bracket(chars: &Vec<char>, start_from: usize) -> usize {
    let mut i = start_from;

    while chars[i] != ']' {
        i += 1;
    }

    i
}

/// Removes single-line comments from every line of the code. Single-line
/// comments begin with a '#'.
fn remove_comments(string: &String) -> String {
    let mut output = String::new();

    for line in string.lines() {
        let filtered_line = if line.contains("#") {
            &line[0..line.find("#").unwrap()]
        } else {
            line
        };

        output.push_str(filtered_line);
    }

    output
}

fn handle_operation(c: char, memory: &mut Memory) -> Option<IdxOp> {
    match c {
        '<' => memory.move_left(),
        '>' => memory.move_right(),
        '+' => memory.inc(),
        '-' => memory.dec(),
        ',' => {
            let input = get_input();
            memory.set(input);
        }
        '.' => {
            let c = memory.get() as char;

            // TODO: Make this slightly better. You'll know what to do.
            if 'A' <= c && c <= 'Z' || 'a' <= c && c <= 'z' {
                println!("{}", c)
            } else {
                println!("{}", memory.get());
            }
        },
        '[' => {
            if memory.get() == 0 {
                return Some(IdxOp::MoveIdx);
            } else {
                return Some(IdxOp::SaveIdx);
            }
        }
        ']' => return Some(IdxOp::BackIdx),
        // TODO: DO THIS SHIT
        '~' => (),
        // Do nothing when any other character appears.
        _ => (),
    }

    None
}

fn get_input() -> u8 {
    let mut input = String::new();

    print!("> ");
    let _ = std::io::stdout().flush();

    let _ = std::io::stdin()
        .read_line(&mut input)
        .expect("couldn't read input");

    // WARNING: Right now, on incorrect input, the program will just crash. This
    // is undesired. This needs to be fixed later on.
    let number: u8 = input.trim().parse().expect("could not convert to number");

    number
}

struct Memory {
    memory: *mut u8,
    size: usize,
    ptr: usize,
}

impl Memory {
    const MIN_SIZE: usize = 8;

    /// Create a new `Memory`.
    ///
    /// # Parameters
    ///
    /// - `size`: Size of the memory in bytes. Cannot be less than
    /// [MIN_SIZE](Self::MIN_SIZE).
    ///
    /// # Panics
    ///
    /// If `size` is set to anything less than [MIN_SIZE](Self::MIN_SIZE),
    /// this function panics.
    fn new(size: usize) -> Self {
        if size < Self::MIN_SIZE {
            panic!("size too small");
        }

        let layout = Layout::array::<u8>(size).unwrap();
        let memory = unsafe { alloc::alloc_zeroed(layout) };

        Self {
            memory,
            size,
            ptr: 0,
        }
    }

    /// Get the value in the current cell.
    fn get(&self) -> u8 {
        unsafe { *(self.memory.wrapping_add(self.ptr)) }
    }

    /// Directly set the value into the current cell.
    fn set(&mut self, value: u8) {
        unsafe {
            *self.memory.wrapping_add(self.ptr) = value;
        }
    }

    /// Move the memory pointer one cell to the left.
    fn move_left(&mut self) {
        if self.ptr == 0 {
            self.ptr = self.size - 1;
            return;
        }

        self.ptr -= 1;
    }

    /// Move the memory pointer one cell to the right.
    fn move_right(&mut self) {
        if self.ptr == self.size - 1 {
            self.ptr = 0;
            return;
        }

        self.ptr += 1;
    }

    /// Increment the value at the current memory cell by 1.
    fn inc(&mut self) {
        unsafe {
            *self.memory = (*self.memory).wrapping_add(1);
        }
    }

    /// Decrement the value at the current memory cell by 1.
    fn dec(&mut self) {
        unsafe {
            *self.memory = (*self.memory).wrapping_sub(1);
        }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        let layout = Layout::array::<u8>(self.size).unwrap();
        unsafe { alloc::dealloc(self.memory, layout); }
    }
}

fn read_file(filepath: &str, into: &mut String) -> io::Result<usize> {
    let mut file = File::open(filepath)
        .expect(&format!("could not open \"{}\"", filepath));

    file.read_to_string(into)
}

/// TODO: Docs.
#[derive(Copy, Clone, Debug, PartialEq)]
enum IdxOp {
    /// This informs the caller that it should save the current index to its
    /// buffer.
    SaveIdx,
    /// This informs the caller that it should move the pointer forward to a
    /// matching ']'.
    MoveIdx,
    /// This informs the caller that is should mvoe the pointer back to the
    /// beginning of the loop, i.e. the matching '['.
    BackIdx,
}

#[allow(dead_code)]
mod debugging {
    use super::Memory;

    // TODO: This might not be used...
    struct OperationData<'a> {
        pub memory: &'a Memory,
        pub i: usize,
        pub c: char,
    }

    /// TODO: Docs.
    pub fn print_memory_block(start: usize, end: usize, memory: &Memory) {
        for i in start..end {
            print!("[{}]", unsafe { *memory.memory.wrapping_add(i) });
        }

        println!();
    }
}
