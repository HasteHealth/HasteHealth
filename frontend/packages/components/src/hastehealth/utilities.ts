export function generateTailwindColorFromValue(value: string) {
  const colors = ["red", "yellow", "green", "blue", "indigo", "purple", "pink"];

  // Generate tailwind color using value
  const index = value.charCodeAt(0) % colors.length;

  return colors[index];
}
