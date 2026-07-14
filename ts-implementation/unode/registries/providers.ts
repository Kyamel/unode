import type { ProviderDefinition, ProviderContext } from '../api/contracts';

export type RegisteredProvider = ProviderDefinition<any, unknown, unknown> & { pluginId: string };

export class ProviderRegistry {
  private providers: RegisteredProvider[] = [];

  register(def: ProviderDefinition<any, unknown, unknown>, pluginId: string) {
    this.providers.push({ ...def, pluginId });
  }

  async provide<TInput, TOutput>(capability: string, input: TInput, ctx: ProviderContext): Promise<TOutput[]> {
    const outputs: TOutput[] = [];
    for (const provider of this.providers) {
      if (provider.capability !== capability) continue;
      const result = await provider.provide(input, ctx);
      outputs.push(result as TOutput);
    }
    return outputs;
  }
}
