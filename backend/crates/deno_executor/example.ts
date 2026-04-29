export {};

interface Person {
  name: string;
  age: number;
}

function hello(t: Person) {
  console.log(`Hello, ${t.name}! You are ${t.age} years old.`);
}

hello({ name: "Alice", age: 30 });
hello({ name: "Bob", age: 25 });
hello({ name: "Charlie", age: 35 });
