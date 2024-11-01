const subscriptionIds = [
  "256e1dbc00482ddd619c293cc0df94d366afe7980022bb22d99e33036fd465dd",
  "434021b6e50e9ee255c40a66e65ba6ac41d8aacde231a263243d123714dbf67f",
  "59ca93bbbf08be7d469596847511a4d066a2c9298ce29624357baa198cc23a0b",
  "673ce04da4b8ed773ee7cd5828a2083775bea4130498b847c5b34b2ed913b07f",
  "69705f85ac4c9fef6c02b4d83426aaaf05154c645ec1c61665f8e450f0468bc0",
  "82c880daad4d0c3a6acfa0c29a79f4dafc53ce8e4624156b0f6164f3d3cb9d04",
  "af5ce56be024e99c2db8cde475b75ff1ddd8e7aa4dc95ae5db1061ed652e264a",
  "dad01517c7a3c428ea903253a9e59964e8db06d323a9bd3f4c74d6366832bdbf",
  "e54bdb17d41d5dbe16418f96b70e44546ccd63e6f3733ae3c192043548998ff3",
];

// Function to get a random subscription ID
export function getSubscriptionId(): string {
  const randomIndex = Math.floor(Math.random() * subscriptionIds.length);
  return subscriptionIds[randomIndex];
}
