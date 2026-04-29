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
