import { DI, newInstanceOf, resolve } from 'aurelia';
import { IHttpClient, json } from '@aurelia/fetch-client';
import type { FieldError, Note, NoteDraft } from './core';

export const INotesApi = DI.createInterface<INotesApi>('INotesApi', x => x.singleton(NotesApi));
export type INotesApi = Pick<NotesApi, 'list' | 'create' | 'remove'>;

/** The server rejected a draft. Carries the same errors the form shows. */
export class InvalidNoteError extends Error {
  constructor(readonly errors: FieldError[]) {
    super(errors.map(error => error.message).join(' '));
    this.name = 'InvalidNoteError';
  }
}

export class NotesApi {
  private readonly http = resolve(newInstanceOf(IHttpClient));

  constructor() {
    this.http.configure(config => config
      .withBaseUrl('/api/')
      .withDefaults({ headers: { Accept: 'application/json' } }));
  }

  async list(): Promise<Note[]> {
    return this.read(await this.http.get('notes'));
  }

  async create(draft: NoteDraft): Promise<Note> {
    const response = await this.http.post('notes', json(draft));
    if (response.status === 422) {
      const body: { errors: FieldError[] } = await response.json();
      throw new InvalidNoteError(body.errors);
    }
    return this.read(response);
  }

  async remove(id: number): Promise<void> {
    const response = await this.http.delete(`notes/${id}`);
    if (!response.ok && response.status !== 404) {
      throw new Error(`DELETE notes/${id} failed with ${response.status}`);
    }
  }

  private async read<T>(response: Response): Promise<T> {
    if (!response.ok) {
      throw new Error(`${response.url} failed with ${response.status}`);
    }
    return response.json() as Promise<T>;
  }
}
