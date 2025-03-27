const originalString = `【replace here】`;
const base64String = Buffer.from(originalString).toString('base64');
console.log(base64String);
