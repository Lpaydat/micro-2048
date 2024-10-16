function bigintToBoard(bigint) {
  // Convert the bigint to a hexadecimal string, padded to 16 characters
  let hexString = bigint.toString(16).padStart(16, "0");

  // Initialize an empty board
  let board = [];

  // Iterate over each row (4 rows total)
  for (let i = 0; i < 4; i++) {
    // Extract 4 hex digits for each row
    let rowHex = hexString.slice(i * 4, (i + 1) * 4);

    // Convert each hex digit to a decimal number and calculate the tile value
    let row = Array.from(rowHex).map((hex) => {
      let value = parseInt(hex, 16);
      return value > 0 ? 2 << (value - 1) : 0;
    });

    // Add the row to the board
    board.push(row);
  }

  return board;
}

// Example usage:
let board = bigintToBoard(0x00000000221100n);
console.log(board);
