/**
 * Bundled 4:3 country flags for the language picker.
 * Imported as hashed assets so the app stays fully offline.
 */
import am from 'flag-icons/flags/4x3/am.svg';
import az from 'flag-icons/flags/4x3/az.svg';
import bg from 'flag-icons/flags/4x3/bg.svg';
import by from 'flag-icons/flags/4x3/by.svg';
import cz from 'flag-icons/flags/4x3/cz.svg';
import de from 'flag-icons/flags/4x3/de.svg';
import dk from 'flag-icons/flags/4x3/dk.svg';
import ee from 'flag-icons/flags/4x3/ee.svg';
import es from 'flag-icons/flags/4x3/es.svg';
import fi from 'flag-icons/flags/4x3/fi.svg';
import fr from 'flag-icons/flags/4x3/fr.svg';
import gb from 'flag-icons/flags/4x3/gb.svg';
import ge from 'flag-icons/flags/4x3/ge.svg';
import gr from 'flag-icons/flags/4x3/gr.svg';
import hr from 'flag-icons/flags/4x3/hr.svg';
import hu from 'flag-icons/flags/4x3/hu.svg';
import ie from 'flag-icons/flags/4x3/ie.svg';
import it from 'flag-icons/flags/4x3/it.svg';
import kg from 'flag-icons/flags/4x3/kg.svg';
import kz from 'flag-icons/flags/4x3/kz.svg';
import lt from 'flag-icons/flags/4x3/lt.svg';
import lv from 'flag-icons/flags/4x3/lv.svg';
import mt from 'flag-icons/flags/4x3/mt.svg';
import nl from 'flag-icons/flags/4x3/nl.svg';
import pl from 'flag-icons/flags/4x3/pl.svg';
import pt from 'flag-icons/flags/4x3/pt.svg';
import ro from 'flag-icons/flags/4x3/ro.svg';
import ru from 'flag-icons/flags/4x3/ru.svg';
import se from 'flag-icons/flags/4x3/se.svg';
import si from 'flag-icons/flags/4x3/si.svg';
import sk from 'flag-icons/flags/4x3/sk.svg';
import tj from 'flag-icons/flags/4x3/tj.svg';
import tm from 'flag-icons/flags/4x3/tm.svg';
import ua from 'flag-icons/flags/4x3/ua.svg';
import uz from 'flag-icons/flags/4x3/uz.svg';

const FLAGS: Record<string, string> = {
    ru,
    uk: ua,
    be: by,
    en: gb,
    de,
    es,
    fr,
    it,
    pt,
    pl,
    cs: cz,
    sk,
    bg,
    hr,
    sl: si,
    nl,
    da: dk,
    sv: se,
    fi,
    et: ee,
    lv,
    lt,
    ro,
    hu,
    el: gr,
    ga: ie,
    mt,
    kk: kz,
    ky: kg,
    tg: tj,
    tk: tm,
    uz,
    az,
    ka: ge,
    hy: am,
};

export function LocaleFlag({ code, width = 18 }: { code: string; width?: number }) {
    const src = FLAGS[code];
    if (!src) return null;
    return (
        <img
            className="flag"
            src={src}
            width={width}
            height={Math.round((width * 3) / 4)}
            alt=""
            aria-hidden="true"
            draggable={false}
        />
    );
}
