export type TravelSettings = {
    speed: number; // in steps per second
    distance: number; // in steps
};

export type State = {
    ws: WebSocket | null;
    zStage: {
        program_1: TravelSettings;
        program_2: TravelSettings;
    }
};
