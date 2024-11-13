use raug::{prelude::*, signal::PI};

pub fn random(graph: &GraphBuilder, trig: &Node) -> Node {
    let noise = graph.add(NoiseOscillator::new());
    let snh = graph.add(SampleAndHold::default());
    trig.output(0).connect(&snh.input("trig"));
    noise.output(0).connect(&snh.input("in"));
    snh
}

pub fn pick_randomly(graph: &GraphBuilder, trig: &Node, options: &[Node]) -> Node {
    let index = random(graph, trig);
    let index = index * (options.len() + 1) as Float;
    let index = index % options.len() as Float;
    let index = index.cast(SignalType::Int);

    let select = graph.add(Select::<bool>::new(options.len()));
    select
        .input("in")
        .connect(graph.constant(AnySignal::new_bool(true)));
    select.input("index").connect(index);

    let merge = graph.add(Merge::<Float>::new(options.len()));

    let msgs = options
        .iter()
        .map(|_| graph.add(Message::<Float>::new(0.0)))
        .collect::<Vec<_>>();

    for (i, (option, msg)) in options.iter().zip(msgs.iter()).enumerate() {
        msg.input(0).connect(select.output(i as u32));
        msg.input(1).connect(option.output(0));
        merge.input(i as u32).connect(msg.output(0));
    }

    merge
}

pub fn fm_sine_osc(graph: &GraphBuilder, freq: &Node, mod_freq: &Node) -> Node {
    let sr = graph.sample_rate();
    let phase = graph.add(PhaseAccumulator::default());
    let increment = freq / sr;
    phase.input(0).connect(increment.output(0));
    (phase * 2.0 * PI + mod_freq * 2.0 * PI).sin()
}

pub fn decay_env(graph: &GraphBuilder, trig: &Node, decay: &Node) -> Node {
    let sr = graph.sample_rate();
    let time = graph.add(PhaseAccumulator::default());
    time.input(0).connect(sr.recip().output(0));
    time.input(1).connect(trig.output(0));

    let time = time % 1.0;

    let env = (-&time + 1.0).powf(decay.recip());

    env.smooth(0.001)
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
    rates: &[Float],
    ratios: &[Float],
    freqs: &[Float],
    decays: &[Float],
    amps: &[Float],
) -> Node {
    let mast = graph.add(Metro::default());
    mast.input(0).connect(rates[0]);

    // select a random rate
    let rates = rates.iter().map(|&r| graph.constant(r)).collect::<Vec<_>>();
    let rate = pick_randomly(graph, &mast, &rates);

    let trig = graph.add(Metro::default());
    trig.input(0).connect(rate.output(0));

    // select a random frequency
    let freqs = freqs.iter().map(|&f| graph.constant(f)).collect::<Vec<_>>();
    let freq = pick_randomly(graph, &trig, &freqs);

    // select a random decay
    let amp_decays = decays
        .iter()
        .map(|&d| graph.constant(d))
        .collect::<Vec<_>>();
    let amp_decay = pick_randomly(graph, &trig, &amp_decays);

    // select a random mod ratio
    let ratios = ratios
        .iter()
        .map(|&r| graph.constant(r))
        .collect::<Vec<_>>();
    let ratio = pick_randomly(graph, &trig, &ratios);

    // select a random amplitude
    let amps = amps.iter().map(|&a| graph.constant(a)).collect::<Vec<_>>();
    let amp = pick_randomly(graph, &trig, &amps);

    // create the amplitude envelope
    let amp_env = decay_env(graph, &trig, &amp_decay);

    // select a random decay
    let filt_decays = decays
        .iter()
        .map(|&d| graph.constant(d))
        .collect::<Vec<_>>();
    let filt_decay = pick_randomly(graph, &trig, &filt_decays);

    // create the filter envelope
    let filt_env = decay_env(graph, &trig, &filt_decay);

    // select a random scale
    let scales = [0.25, 0.5, 1.0];
    let scales = scales
        .iter()
        .map(|&s| graph.constant(s))
        .collect::<Vec<_>>();
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

    let amp = graph.add_param(Param::<Float>::new("amp", Some(0.5)));

    let mut tones = vec![];
    for i in 0..num_tones {
        let freqs = scale_freqs(i as Float * 0.00);
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
