export const tools = {
  hello_plugin_greet: async ({ args }) => ({ greeting: `Hello, ${args.name ?? "world"}!` })
};

