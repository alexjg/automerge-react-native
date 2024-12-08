import { greet } from './codegen';

export function sayHello() {
  console.log("Message from native:", greet());
}
