/*
 * Main module for the Kafkakitty client side code
 */

console.log('😺 Kafkakitty bootstrapping..');

const socket = new WebSocket('ws://127.0.0.1:8001');

socket.onmessage = (event) => {
  console.log(`Received: ${event}`);
};
