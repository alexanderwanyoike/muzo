import type {
  PlaylistDto,
  PlaylistService,
} from "../interfaces/playlist-interfaces";
import type { CommandHandler } from "./command-handler";
import type { Result } from "../result";
import { parseInput, parseNonEmptyString } from "./command-arguments";

export class CreatePlaylistCommand implements CommandHandler {
  readonly command = "create_playlist";

  constructor(private readonly service: PlaylistService) {}

  async handle(args: unknown): Promise<Result<PlaylistDto>> {
    const input = parseInput(args, this.command);
    if (!input.ok) return input;
    const name = parseNonEmptyString(input.value, "name", this.command);
    if (!name.ok) return name;
    return this.service.create(name.value);
  }
}
