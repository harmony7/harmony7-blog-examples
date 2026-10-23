// The resource's representation: an ordinary JavaScript class.
class Tally {
  constructor(name) {
    this.name = name;
    this.total = 0;
    console.error(`tally: created ${name}`);
  }

  add(n) {
    this.total += n;
    return this.total;
  }

  // Called when the last handle is dropped.
  [Symbol.dispose]() {
    console.error(`tally: dropped ${this.name} at ${this.total}`);
  }
}

export const tally = { Tally };
