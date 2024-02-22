#![feature(let_chains)]
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum Tile {
    Latent(usize),
    Active(usize),
    Dead,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct SeenState {
    tiles: Vec<Tile>,
    location: Option<usize>,
    reward: usize,
}

#[derive(Debug, Clone)]
struct GameState {
    tiles: Vec<Tile>,
    location: Option<usize>,
    start_location: Option<usize>,
    threshold: usize,
    reward: usize,
    location_queue: Vec<Option<usize>>,
    action_queue: Vec<Action>,
}

impl GameState {
    fn new(tiles: Vec<Tile>) -> Self {
        Self {
            tiles,
            location: None,
            start_location: None,
            action_queue: vec![],
            location_queue: vec![],
            threshold: 6,
            reward: 0,
        }
    }

    fn step(&mut self) {
        self.tiles.iter_mut().for_each(|t| match t {
            Tile::Latent(0) => {
                *t = Tile::Active(1);
            }
            Tile::Latent(v) => {
                *v -= 1;
            }
            Tile::Active(v) => {
                if *v < self.threshold {
                    *v += 1;
                } else {
                    *t = Tile::Dead;
                }
            }
            _ => {}
        });
    }

    fn execute(&mut self, action: Action) {
        match action {
            Action::Collect => {
                let selected = &mut self.tiles[self.location.expect("Swipe without location")];

                if let Tile::Active(v) = selected {
                    self.reward += (*v).pow(2);
                }

                *selected = Tile::Dead;
            }
            Action::Advance => {
                self.step();
            }
            Action::CounterClockwise => {
                let location = self.location.expect("Move without location");
                // watch out for underflow prior to wrap
                self.location = Some((location + self.tiles.len() - 1) % self.tiles.len());
                self.step();
            }
            Action::Clockwise => {
                let location = self.location.expect("Move without location");
                self.location = Some((location + 1) % self.tiles.len());
                self.step();
            }
        }
        self.action_queue.push(action);
        self.location_queue.push(self.location);
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Advance,
    CounterClockwise,
    Clockwise,
    Collect,
}

// DFS to find max reward
fn solve(state: GameState, seen: &mut std::collections::HashSet<SeenState>) -> Option<GameState> {
    let mut best_state: Option<GameState> = None;

    if state.tiles.iter().all(|t| matches!(t, Tile::Dead)) {
        return Some(state);
    } else if seen.contains(&SeenState {
        tiles: state.tiles.clone(),
        location: state.location,
        reward: state.reward,
    }) {
        return None;
    } else {
        let seenstate = SeenState {
            tiles: state.tiles.clone(),
            location: state.location,
            reward: state.reward,
        };
        seen.insert(seenstate);
    }

    if state.location.is_none() {
        for i in 0..state.tiles.len() {
            let mut new_state = state.clone();
            new_state.location = Some(i);
            new_state.start_location = Some(i);
            new_state.step();
            if let Some(result) = solve(new_state, seen) {
                match best_state {
                    None => best_state = Some(result),
                    Some(ref mut best_state) => {
                        if result.reward > best_state.reward
                            || (result.reward == best_state.reward
                                && result.action_queue.len() < best_state.action_queue.len())
                        {
                            *best_state = result.clone();
                        }
                    }
                }
            }
        }
    } else {
        for action in [
            Action::Advance,
            Action::CounterClockwise,
            Action::Clockwise,
            Action::Collect,
        ] {
            if let Some(Action::Collect) = state.action_queue.last()
                && matches!(action, Action::Collect)
            {
                // don't collect twice in a row
                continue;
            }

            let mut new_state = state.clone();
            new_state.execute(action);

            if let Some(result) = solve(new_state, seen) {
                match best_state {
                    None => best_state = Some(result),
                    Some(ref mut best_state) => {
                        if result.reward > best_state.reward
                            || (result.reward == best_state.reward
                                && result.action_queue.len() < best_state.action_queue.len())
                        {
                            *best_state = result.clone();
                        }
                    }
                }
            }
        }
    }
    best_state
}

fn accumulate(actions: &[Action], start: usize) {
    let mut location = start as isize; // cheat offset here initially
    let mut actions = actions.iter().peekable();
    while let Some(action) = actions.next() {
        match *action {
            Action::Advance => {
                let mut n = 1;
                while matches!(actions.peek(), Some(Action::Advance)) {
                    actions.next();
                    n += 1;
                }
                println!("Tap the active region {n} times");
            }
            Action::CounterClockwise => {
                let mut n = 1;
                while matches!(actions.peek(), Some(Action::CounterClockwise)) && n <= 3 {
                    actions.next();
                    n += 1;
                }
                location -= n;
                location = location.rem_euclid(6);

                if matches!(actions.peek(), Some(Action::Collect)) {
                    actions.next();
                    println!("Swipe on {}", location);
                } else {
                    println!("Tap on {}", location);
                }
            }
            Action::Clockwise => {
                let mut n = 1;
                while matches!(actions.peek(), Some(Action::Clockwise)) && n <= 3 {
                    actions.next();
                    n += 1;
                }
                location += n;
                location = location.rem_euclid(6);

                if matches!(actions.peek(), Some(Action::Collect)) {
                    actions.next();
                    println!("Swipe on {}", location);
                } else {
                    println!("Tap on {}", location);
                }
            }
            Action::Collect => {
                println!("Swipe on active region");
            }
        }
    }
}

fn main() {
    loop {
        println!("Pick one region to be \"0\" and, the rest of the regions are enumerated clockwise from there.");
        println!("Enter the number of latent tiles for each region going clockwise from \"0\" (white trapezoid distance from edge, starting from zero), and - for dead tiles, then press enter.");
        println!("This solver is agnostic to the number of regions, so make sure to be typing exactly 6 characters, if that is what you intend.");
        let mut tiles = Vec::new();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        for c in input.chars() {
            match c.to_digit(10) {
                Some(v) => tiles.push(Tile::Latent(v as usize)),
                None => tiles.push(Tile::Dead),
            }
        }

        let game = GameState::new(tiles);

        let mut seen = std::collections::HashSet::new();
        let best_state = solve(game.clone(), &mut seen).unwrap();
        println!("Number of states: {}", seen.len());
        println!();

        println!("Tap on {} to start", best_state.start_location.unwrap());
        accumulate(&best_state.action_queue, best_state.start_location.unwrap());
        println!();
    }
}
