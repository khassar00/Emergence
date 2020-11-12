use auto_ops::impl_op_ex;
use rand::distributions::{Distribution, Standard, Uniform};
use rand::Rng;

use crate::config::MAP_SIZE;

#[derive(Debug, Clone, Copy)]
pub struct CubePosition {
	pub alpha: isize,
	pub beta: isize,
	pub gamma: isize,
}

impl CubePosition {
	pub fn to_axial(self) -> Position {
		Position {
			alpha: self.alpha,
			beta: self.beta,
		}
	}
}

// We're using a horizontal layout hex grid
// with an "axial coordinate" system
// See: https://www.redblobgames.com/grids/hexagons/
// alpha == q, beta == r from that article

static ORIGIN: Position = Position { alpha: 0, beta: 0 };

const MIRROR_DISTANCE: isize = 2 * MAP_SIZE + 1;

// FIXME: coordinates are wrong. See: https://www.redblobgames.com/grids/hexagons/#wraparound for formula
static MIRROR_CENTERS: [Position; 6] = [
	Position {
		alpha: MIRROR_DISTANCE,
		beta: 0,
	},
	Position {
		alpha: MIRROR_DISTANCE,
		beta: -MIRROR_DISTANCE,
	},
	Position {
		alpha: 0,
		beta: -MIRROR_DISTANCE,
	},
	Position {
		alpha: -MIRROR_DISTANCE,
		beta: 0,
	},
	Position {
		alpha: -MIRROR_DISTANCE,
		beta: MIRROR_DISTANCE,
	},
	Position {
		alpha: 0,
		beta: MIRROR_DISTANCE,
	},
];

#[derive(Debug, Clone, Copy)]
pub struct Position {
	pub alpha: isize,
	pub beta: isize,
}

impl Position {
	pub fn to_cubic(self) -> CubePosition {
		CubePosition {
			alpha: self.alpha,
			beta: self.beta,
			gamma: -self.alpha - self.beta,
		}
	}

	fn wrap(self) -> Position {
		dbg!(self);
		let d = self.dist(ORIGIN);
		dbg!(d);

		if d <= MAP_SIZE {
			return self;
		}

		let mut proposal = self;

		for i in MIRROR_CENTERS.iter() {
			if self.dist(*i) <= MAP_SIZE {
				proposal.alpha = self.alpha - i.alpha;
				proposal.beta = self.beta - i.beta;
				break;
			}
		}

		return proposal;
	}

	pub fn dist(self, b: Position) -> isize {
		let (a, b) = (self.to_cubic(), b.to_cubic());
		((a.alpha - b.alpha).abs() + (a.beta - b.beta).abs() + (a.gamma - b.gamma).abs()) / 2
	}

	pub fn translate(self, direction: &HexDirection, distance: isize) -> Position {
		let proposed_position = self + direction.offset() * distance;

		return proposed_position.wrap();
	}

	fn translate_unchecked(self, direction: &HexDirection, distance: isize) -> Position {
		self + direction.offset() * distance
	}

	pub fn ring(self, radius: isize) -> Vec<Position> {
		let mut positions: Vec<Position> = Vec::new();

		if radius == 0 {
			positions.push(self);
			return positions;
		}

		let mut current_position = self.translate_unchecked(&HexDirection::East, radius);

		let mut current_direction = HexDirection::Southwest;

		for _ in 0..6 {
			for _ in 0..radius {
				positions.push(current_position);
				current_position = current_position.translate_unchecked(&current_direction, 1);
			}

			current_direction = current_direction.rotate(1);
		}
		return positions;
	}

	pub fn hexagon(self, radius: isize) -> Vec<Position> {
		let mut positions: Vec<Position> = Vec::new();

		for i in 0..=radius {
			positions.extend(Position::ring(self, i));
		}

		return positions;
	}
}

impl_op_ex!(+ |a: Position, b: Position| -> Position {
	 Position{
		 alpha: a.alpha + b.alpha,
		 beta: a.beta + b.beta
	}
});

impl_op_ex!(*|a: Position, c: usize| -> Position {
	Position {
		alpha: a.alpha * c as isize,
		beta: a.beta * c as isize,
	}
});

impl_op_ex!(*|c: usize, a: Position| -> Position {
	Position {
		alpha: a.alpha * c as isize,
		beta: a.beta * c as isize,
	}
});

impl_op_ex!(*|a: Position, c: isize| -> Position {
	Position {
		alpha: a.alpha * c,
		beta: a.beta * c,
	}
});

impl_op_ex!(*|c: isize, a: Position| -> Position {
	Position {
		alpha: a.alpha * c,
		beta: a.beta * c,
	}
});

impl_op_ex!(-|a: Position, b: Position| -> Position {
	Position {
		alpha: a.alpha - b.alpha,
		beta: a.beta - b.beta,
	}
});

impl_op_ex!(+= |a: &mut Position, b: &Position| { a.alpha += b.alpha; a.beta += b.beta;});
impl_op_ex!(-= |a: &mut Position, b: &Position| { a.alpha -= b.alpha; a.beta -= b.beta;});

#[derive(Debug)]
pub enum HexDirection {
	East,
	Southeast,
	Southwest,
	West,
	Northwest,
	Northeast,
}

// Generate a random direction with:
// use rand::distributions::Standard;
// let mut rng = &mut rand::thread_rng();
// let direction: HexDirection = rng.sample(Standard);

impl Distribution<HexDirection> for Standard {
	fn sample<R: Rng + ?Sized>(&self, mut rng: &mut R) -> HexDirection {
		let options = Uniform::from(0..5);
		let choice = options.sample(&mut rng);

		HexDirection::from_int(choice)
	}
}

impl HexDirection {
	fn from_int(choice: isize) -> HexDirection {
		let int_direction = choice.rem_euclid(6);
		use HexDirection::*;
		match int_direction {
			0 => East,
			1 => Southeast,
			2 => Southwest,
			3 => West,
			4 => Northwest,
			5 => Northeast,
			_ => unreachable!(),
		}
	}

	fn to_int(self) -> u8 {
		use HexDirection::*;
		match self {
			East => 0,
			Southeast => 1,
			Southwest => 2,
			West => 3,
			Northwest => 4,
			Northeast => 5,
		}
	}
	pub fn offset(&self) -> Position {
		use HexDirection::*;
		match self {
			East => Position { alpha: 1, beta: 0 },
			Southeast => Position { alpha: 1, beta: -1 },
			Southwest => Position { alpha: 0, beta: -1 },
			West => Position { alpha: -1, beta: 0 },
			Northwest => Position { alpha: -1, beta: 1 },
			Northeast => Position { alpha: 0, beta: 1 },
		}
	}

	// Positive steps rotates clockwise, negative steps rotate counterclockwise
	pub fn rotate(self, steps: isize) -> HexDirection {
		HexDirection::from_int(self.to_int() as isize + steps.rem_euclid(6))
	}
}

pub enum ID {
	Empty,
	Ant,
	Plant,
	Fungus,
}

pub enum SignalType {
	Passive(ID),
	Push(ID),
	Pull(ID),
	Work,
}
