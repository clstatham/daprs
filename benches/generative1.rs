use raug::{prelude::*, signal::PI};

pub fn random(graph: &GraphBuilder, trig: &Node) -> Node {
    let noise = graph.add(NoiseOscillator::new());
    let snh = graph.add(SampleAndHold::default());
    trig.output(0).connect(&snh.input("trig"));
    noise.output(0).connect(&snh.input("in"));
    snh
}

pub fn pick_randomly(graph: &GraphBuilder, trig: &Node, options: &Node) -> Node {
    let index = random(graph, trig);
    let len = options.len();
    let index = index * (&len + 1).cast(SignalType::Float);
    let index = index % len.cast(SignalType::Float);
    let index = index.cast(SignalType::Int);

    let get = graph.add(Get::new(SignalType::Float));

    get.input("list").connect(options);
    get.input("index").connect(index);

    get
}

pub fn fm_sine_osc(graph: &GraphBuilder, freq: &Node, mod_freq: &Node) -> Node {
    let sine = graph.add(SineOscillator::default());
    sine.input("frequency").connect(freq);
    let phase = mod_freq * 2.0 * PI;
    sine.input("phase").connect(phase);
    sine
}

pub fn decay_env(graph: &GraphBuilder, trig: &Node, decay: &Node) -> Node {
    let env = graph.add(DecayEnv::default());
    env.input("tau").connect(decay);
    env.input("trig").connect(trig);
    env
}

pub fn midi_to_freq(midi: Float) -> Float {
    440.0 * Float::powf(2.0, (midi - 69.0) / 12.0)
}

pub fn scale_freqs(detune: Float) -> Vec<Float> {
    // minor scale
    let scale = [0, 2, 3, 5, 7, 8, 10];
    let base = 60; // C4
    let mut freqs = vec![];
    for note in &scale {
        freqs.push(midi_to_freq(base as Float + *note as Float + detune));
    }
    let base = 72;
    for note in &scale {
        freqs.push(midi_to_freq(base as Float + *note as Float + detune));
    }
    let base = 48;
    for note in &scale {
        freqs.push(midi_to_freq(base as Float + *note as Float + detune));
    }
    freqs
}

pub fn random_tones(
    graph: &GraphBuilder,
    rates: &Node,
    ratios: &Node,
    freqs: &Node,
    decays: &Node,
    amps: &Node,
) -> Node {
    let mast = graph.add(Metro::default());
    let get = graph.add(Get::new(SignalType::Float));
    get.input("list").connect(rates);
    get.input("index").connect(0);
    mast.input("period").connect(get);

    // select a random rate
    let rate = pick_randomly(graph, &mast, rates);

    let trig = graph.add(Metro::default());
    trig.input("period").connect(rate);

    // select a random frequency
    let freq = pick_randomly(graph, &trig, freqs);

    // select a random decay
    let amp_decay = pick_randomly(graph, &trig, decays);

    // select a random mod ratio
    let ratio = pick_randomly(graph, &trig, ratios);

    // select a random amplitude
    let amp = pick_randomly(graph, &trig, amps);

    // create the amplitude envelope
    let amp_env = decay_env(graph, &trig, &amp_decay);

    // select a random decay
    let filt_decay = pick_randomly(graph, &trig, decays);

    // create the filter envelope
    let filt_env = decay_env(graph, &trig, &filt_decay);

    // select a random scale
    let scales = [0.25, 0.5, 1.0];
    let scales = graph.constant(SignalBuffer::from_iter(scales.iter().copied()));
    let scale = pick_randomly(graph, &trig, &scales);

    // scale the filter envelope
    let filt_env = filt_env * scale * 19800.0 + 200.0;

    // create the modulator
    let modulator = graph.add(BlSawOscillator::default());
    modulator.input(0).connect((&freq * ratio).output(0));

    // create the carrier
    let carrier = fm_sine_osc(graph, &freq, &(modulator * 0.1));

    // create the filter
    let filt = graph.add(MoogLadder::default());
    filt.input("in").connect(carrier.output(0));
    filt.input("cutoff").connect(filt_env.output(0));
    filt.input("resonance").connect(0.1);

    filt * amp_env * amp
}

pub fn generative1(num_tones: usize) -> GraphBuilder {
    let ratios = [0.25, 0.5, 1.0, 2.0];
    let decays = [0.02, 0.1, 0.2, 0.5];
    let amps = [0.125, 0.25, 0.5, 0.8];
    let rates = [1. / 8., 1. / 4., 1. / 2., 1., 2.];

    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let amp = graph
        .add_param(Param::new::<Float>("amp", Some(0.5)))
        .make_register();

    let ratios = graph.constant(SignalBuffer::from_iter(ratios.iter().copied()));
    let decays = graph.constant(SignalBuffer::from_iter(decays.iter().copied()));
    let amps = graph.constant(SignalBuffer::from_iter(amps.iter().copied()));
    let rates = graph.constant(SignalBuffer::from_iter(rates.iter().copied()));

    let freqs = scale_freqs(0.0);
    let freqs = graph.constant(SignalBuffer::from_iter(freqs.iter().copied()));

    let mut tones = vec![];
    for _ in 0..num_tones {
        let tone = random_tones(&graph, &rates, &ratios, &freqs, &decays, &amps);
        tones.push(tone);
    }

    let mut mix = tones[0].clone();
    for tone in tones.iter().skip(1) {
        mix = mix.clone() + tone.clone();
    }

    let mix = mix * amp;

    let master = graph.add(PeakLimiter::default());
    master.input(0).connect(mix.output(0));

    master.output(0).connect(&out1.input(0));
    master.output(0).connect(&out2.input(0));

    graph
}
