import type { CommandContext, CommandDefinition, CommandResult } from '../api/contracts';

type LazyText = string | (() => string);

type RegisteredCommand = Omit<CommandDefinition<any>, 'title' | 'category'> & {
  pluginId: string;
  title: LazyText;
  category?: LazyText;
};

export type ResolvedCommand = Omit<CommandDefinition<any>, 'title' | 'category'> & {
  pluginId: string;
  title: string;
  category?: string;
};

export class CommandRegistry {
  private commands: RegisteredCommand[] = [];

  register(
    def: Omit<CommandDefinition<any>, 'title' | 'category'> & {
      title: LazyText;
      category?: LazyText;
    },
    pluginId: string
  ) {
    this.commands.push({ ...def, pluginId });
  }

  async getAvailable(ctx: CommandContext): Promise<ResolvedCommand[]> {
    const available: ResolvedCommand[] = [];
    for (const command of this.commands) {
      const allowed = command.when ? await command.when(ctx) : true;
      if (allowed) {
        available.push({
          ...command,
          title: typeof command.title === 'function' ? command.title() : command.title,
          category:
            typeof command.category === 'function'
              ? command.category()
              : command.category
        });
      }
    }
    return available;
  }

  async run(id: string, ctx: CommandContext): Promise<CommandResult | void> {
    const command = this.commands.find((entry) => entry.id === id);
    if (!command) throw new Error(`Command not found: ${id}`);
    const allowed = command.when ? await command.when(ctx) : true;
    if (!allowed) throw new Error(`Command not available: ${id}`);
    return await command.run(ctx);
  }
}
