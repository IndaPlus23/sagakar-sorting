use std::io::{stdout, stdin, Write};
use std::thread::sleep;
use std::time::Duration;
use std::collections::VecDeque;
use rand::seq::SliceRandom;
use rand::thread_rng;
use crossterm::{
    execute,
    queue,
    terminal::{self, Clear, ClearType, SetSize},
    cursor::{self, MoveTo, MoveUp, MoveLeft, MoveToRow, MoveToColumn},
    style::{SetForegroundColor, ResetColor, Color}
};
use rodio::{
    self,
    source::{Source, SineWave},
    OutputStream,
    OutputStreamHandle
};

const ELEMENTS: u16 = 100;
const HEIGHT: u16 = ELEMENTS / 4 + 1;

const SLEEP_MILLIS: u64 = 10;
const SLEEP_DURATION: Duration = Duration::from_millis(SLEEP_MILLIS);
const AUDIO_DURATION: Duration = Duration::from_millis(SLEEP_MILLIS);

const BORDER_SIZE: u16 = 1;

fn main() {
    println!("Choose a sorting algorithm to visualise");
    println!("1: Insertion sort");
    println!("2: Selection sort");
    println!("3: Merge sort");
    println!("4: Cocktail shaker sort");
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    let (_stream, audio_handle) = OutputStream::try_default().expect("No output device found");
    let visualizer = Visualizer::new(audio_handle);

    // Create random list to sort
    let mut list = create_random_list();

    match input.trim() {
        "1" => {
            visualizer.setup_terminal(&list);
            insertion_sort(&mut list, Some(visualizer.clone()));
        },
        "2" => {
            visualizer.setup_terminal(&list);
            selection_sort(&mut list, Some(visualizer.clone()));
        },
        "3" => {
            visualizer.setup_terminal(&list);
            merge_sort(&mut list, Some(visualizer.clone()));
        },
        "4" => {
            visualizer.setup_terminal(&list);
            cocktail_sort(&mut list, Some(visualizer.clone()));
        }
        _ => {
            println!("Invalid choice, quitting...");
            std::process::exit(1);
        }
    }
    visualizer.print_list(&list);

    terminal::disable_raw_mode().expect("Failed to unraw terminal");
    execute!(stdout(), cursor::Show, MoveTo(0, HEIGHT + 1)).expect("Failed writing to stdout");
    println!("Press any key to exit...");
    stdin().read_line(&mut input).unwrap();
    std::process::exit(0);
}

// Struct for visualizing and sonicizing sorting algorithms
// Including an instance of this struct when running an algorithm will enable visualization
// Contains methods for playing audio and drawing visualizations to the terminal
#[derive(Clone)]
struct Visualizer {
    handle: OutputStreamHandle, // Allows playing audio
    x_offset: u16, // For merge sort
}
impl Visualizer {

    // Creates a new instance
    fn new(handle: OutputStreamHandle) -> Visualizer {
        return Visualizer { handle: handle, x_offset: 0 }
    }

    fn set_x_offset(&mut self, x: u16) {
        self.x_offset = x;
    }

    fn get_x_offset(&self) -> u16{
        return self.x_offset;
    }

    // Sets the terminal up for visualization
    // Clears buffer, hides cursor and enables raw mode
    fn setup_terminal(&self, list: &Vec<u16>) {
        let mut out = stdout();
        out.flush().expect("Failed writing to stdout");
        execute!(
            out,
            cursor::Hide,
            MoveTo(0, 0),
            SetSize(ELEMENTS + 2 * BORDER_SIZE, HEIGHT + 2 * BORDER_SIZE),
            Clear(ClearType::All),
        ).expect("Failed writing to stdout");
        terminal::enable_raw_mode().expect("Failed to raw terminal");
        self.print_list(list);
    }

    // Plays a frequency proportional to the value passed
    // Used for sonicizing the currently selected value
    fn value_to_sound(&self, value: &u16) {
        let source = SineWave::new((value * 10) as f32).take_duration(AUDIO_DURATION);
        self.handle.play_raw(source).expect("Failed playing audio");
    }

    // Prints a value as a stack of colored blocks to the given terminal x coordinate
    // Used to visualize the list to be sorted, and to partially redraw it while sorting
    fn print_stack(&self, x: u16, value: u16, color: Color) {
        let mut out = stdout();
        execute!(out, MoveToColumn(x + BORDER_SIZE), SetForegroundColor(color)).expect("Failed writing to stdout");
        let mut remaining = value;
        for _i in 0..HEIGHT {
            let character: &str = match remaining {
                0 => {" "},
                1 => {
                    remaining -= 1;
                    "▂"
                },
                2 => {
                    remaining -= 2;
                    "▄"
                },
                3 => {
                    remaining -= 3;
                    "▆"
                },
                _ => {
                    remaining -= 4;
                    "█"
                }
            };
            out.write(character.as_bytes()).expect("Failed writing to stdout");
            queue!(out, MoveUp(1), MoveLeft(1)).expect("Failed writing to stdout");
        }
        queue!(out, MoveToRow(HEIGHT), ResetColor).expect("Failed writing to stdout");
        out.flush().expect("Failed writing to stdout");
    }

    // Visualizes the entire list as stacks of white blocks
    // Be careful with usage, overly frequent redraws of the whole list tanks performance
    fn print_list(&self, list: &Vec<u16>) {
        execute!(stdout(), MoveToRow(HEIGHT)).expect("Failed writing to stdout");
        let mut x = 0;
        for item in list {
            self.print_stack(x, *item, Color::White);
            x += 1;
        }
    }

}

// Generates a randomized list with values ranging from 1 to ELEMENTS
fn create_random_list() -> Vec<u16>{
    let mut list: Vec<u16> = (1..ELEMENTS + 1).collect();
    list.shuffle(&mut thread_rng());
    return list
}

// Sorts a list in place using insertion sort
// Visualizes the sorting if a Visualizer instance is provided
fn insertion_sort(list: &mut Vec<u16>, visualizer: Option<Visualizer>) {
    for i in 1..list.len() {
        let x = list[i];
        let mut j = i16::try_from(i - 1).unwrap();
        while j >= 0 && list[usize::try_from(j).unwrap()] > x {
            let unsigned_j = usize::try_from(j).unwrap();
            list[unsigned_j + 1] = list[unsigned_j];
            if let Some(vis) = &visualizer {
                vis.print_stack((j + 1) as u16, list[unsigned_j + 1], Color::Red);
                vis.value_to_sound(&list[unsigned_j + 1]);
                if j + 2 < list.len() as i16 {
                    vis.print_stack((j + 2) as u16, list[unsigned_j + 2], Color::White);
                }
                sleep(SLEEP_DURATION);
            }
            
            j -= 1;   
        }
        list[usize::try_from(j + 1).unwrap()] = x;
        if let Some(vis) = &visualizer {
            vis.print_list(list);
        }
    }
}

// Sorts a list in place using selection sort
// Visualizes the sorting if a Visualizer instance is provided
fn selection_sort(list: &mut Vec<u16>, visualizer: Option<Visualizer>) {
    for i in 0..(list.len() - 1) {
        let mut min_index = i;
        for j in (i + 1)..list.len() {
            if list[j] < list [min_index] {
                min_index = j;
            }
            if let Some(vis) = &visualizer {
                vis.print_stack(u16::try_from(j - 1).unwrap(), list[j - 1], Color::White);
                vis.print_stack(u16::try_from(j).unwrap(), list[j], Color::Red);
                vis.value_to_sound(&list[j]);
                sleep(SLEEP_DURATION);
            }
        }
        if min_index != i {
            let temp = list[i];
            list[i] = list[min_index];
            list[min_index] = temp;
        }
        if let Some(vis) = &visualizer {
            vis.print_list(list);
        }    
    }
}

// Sorts a list in place using merge sort
// Visualizes the sorting if a Visualizer instance is provided
fn merge_sort(list: &mut Vec<u16>, visualizer: Option<Visualizer>) {
    if list.len() == 1 {
        return;
    }

    let mut left = list[0..(list.len() / 2)].to_owned();
    let mut right = list[(list.len() / 2)..list.len()].to_owned();

    if let Some(vis) = &visualizer {
        merge_sort(&mut left, Some(vis.clone()));
        let mut right_vis = vis.clone();
        right_vis.set_x_offset(vis.get_x_offset() + (list.len() / 2) as u16);
        merge_sort(&mut right, Some(right_vis));
    }
    else {
        merge_sort(&mut left, None);
        merge_sort(&mut right, None);
    }

    merge(left, right, list, visualizer);
}

// Merges two sorted lists into one sorted list
// Visualizes the merge process if provided with a Visualizer instance
fn merge(left: Vec<u16>, right: Vec<u16>, list: &mut Vec<u16>, visualizer: Option<Visualizer>) {
    let mut left = VecDeque::from(left);
    let mut right = VecDeque::from(right);
    let mut index = 0;

    // Add the smallest element from the front of left/right until one runs out
    while left.len() > 0 && right.len() > 0 {
        if left[0] < right[0] {
            list[index] = left.pop_front().unwrap();
        }
        else {
            list[index] = right.pop_front().unwrap();
        }
        if let Some(vis) = &visualizer {
            merge_sweep(index, list, vis);
            vis.value_to_sound(&list[index]);
            sleep(SLEEP_DURATION);
        }
        index += 1;
    }

    // Make sure both left and right are depleted
    while left.len() > 0 {
        list[index] = left.pop_front().unwrap();
        if let Some(vis) = &visualizer {
            merge_sweep(index, list, vis);
            vis.value_to_sound(&list[index]);
            sleep(SLEEP_DURATION);
        }
        index += 1;
    }
    while right.len() > 0 {
        list[index] = right.pop_front().unwrap();
        if let Some(vis) = &visualizer {
            merge_sweep(index, list, vis);
            vis.value_to_sound(&list[index]);
            sleep(SLEEP_DURATION);
        }
        index += 1;
    }
    if let Some(vis) = &visualizer {
        vis.print_stack((index - 1) as u16 + vis.get_x_offset(), list[index - 1], Color::White);
    }   
}

// Colors the stack at current x red, and resets the one behind it to white
// For use in the merge function
fn merge_sweep(index: usize, list: &Vec<u16>, visualizer: &Visualizer) {
    let x = index as u16 + visualizer.get_x_offset();
    visualizer.print_stack(x, list[index], Color::Red);
    if index > 0 {
        visualizer.print_stack(x - 1, list[index - 1], Color::White);
    }
}

// Sorts the list using the cocktail shaker sort algorithm
// Visualizes the sorting if an instance of Visualizer is provided
// Implementation based on pseudocode found here:
// https://en.wikipedia.org/wiki/Cocktail_shaker_sort#Pseudocode
fn cocktail_sort(list: &mut Vec<u16>, visualizer: Option<Visualizer>) {
    // Will remain true as long as two elements are swapped in an iteration
    let mut lower_bound = 0;
    let mut upper_bound = list.len() - 1;
    while lower_bound <= upper_bound {
        for i in lower_bound..upper_bound {
            if list[i] > list[i + 1] {
                let temp = list[i];
                list[i] = list[i + 1];
                list[i + 1] = temp;
                upper_bound = i;
            }
            if let Some(vis) = &visualizer {
                vis.print_stack((i + 1) as u16, list[i + 1], Color::Red);
                vis.value_to_sound(&list[i + 1]);
                vis.print_stack(i as u16, list[i], Color::White);
                sleep(SLEEP_DURATION);
            }
        }
        if let Some(vis) = &visualizer {
            vis.print_list(list)
        }
        for i in (lower_bound..upper_bound).rev() {
            if list[i] > list[i + 1] {
                let temp = list[i];
                list[i] = list[i + 1];
                list[i + 1] = temp;
                lower_bound = i;
            }
            if let Some(vis) = &visualizer {
                vis.print_stack(i as u16, list[i], Color::Red);
                vis.value_to_sound(&list[i]);
                vis.print_stack((i + 1) as u16, list[i + 1], Color::White);
                sleep(SLEEP_DURATION);
            }
            
        }
        if let Some(vis) = &visualizer {
            vis.print_list(list)
        }
        lower_bound += 1;
    }
    

}


// Unit tests
// Tests that all sorting algorithms produce the same results as the built-in sorting method
#[cfg(test)]
mod tests {
    use crate::{create_random_list, insertion_sort, selection_sort, merge_sort, cocktail_sort};

    #[test]
    fn test_insertion() {
        let mut std_sorted = create_random_list();
        std_sorted.sort();

        let mut insertion_sorted = create_random_list();
        insertion_sort(&mut insertion_sorted, None);
        
        assert_eq!(std_sorted, insertion_sorted);
    }

    #[test]
    fn test_selection() {
        let mut std_sorted = create_random_list();
        std_sorted.sort();

        let mut selection_sorted = create_random_list();
        selection_sort(&mut selection_sorted, None);

        assert_eq!(std_sorted, selection_sorted);
    }

    #[test]
    fn test_merge() {
        let mut std_sorted = create_random_list();
        std_sorted.sort();

        let mut merge_sorted = create_random_list();
        merge_sort(&mut merge_sorted, None);

        assert_eq!(std_sorted, merge_sorted);
    }

    #[test]
    fn test_cocktail() {
        let mut std_sorted = create_random_list();
        std_sorted.sort();

        let mut cocktail_sorted = create_random_list();
        cocktail_sort(&mut cocktail_sorted, None);

        assert_eq!(std_sorted, cocktail_sorted);
    }
}