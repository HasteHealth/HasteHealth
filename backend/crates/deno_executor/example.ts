// const obj = new InteropObject(22);

// console.log(obj, obj.value);
// obj.value = 99;
// console.log(obj.value);

// const z = await InteropObject.create(88);

// console.log(z.value);

const patient = await readResource("Patient", "90277570");

console.log(patient.id);
console.log(patient.name);

export {};

interface Person {
  name: string;
  age: number;
}

function hello(t: Person) {
  console.log(`Hello, ${t.name}! You are ${t.age} years old.`);
}

// hello({ name: "Alice", age: 30 });
// hello({ name: "Bob", age: 25 });
// hello({ name: "Charlie", age: 35 });
