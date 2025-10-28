export interface AppState {
	themes: Array<string>;
	syntaxes: Array<string>;
}

export interface SettingsPageState {
	app: AppState;
	visible: boolean;
	visited: boolean;
}
