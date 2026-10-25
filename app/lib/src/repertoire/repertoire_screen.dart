import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/l10n/generated/app_localizations.dart';

/// Lists the pieces and loads new ones from files (US 2); pops with the id of
/// the chosen or loaded piece.
class RepertoireScreen extends StatefulWidget {
  const RepertoireScreen({
    super.key,
    required this.engine,
    this.pickFile = pickScoreFile,
  });

  final EngineClient engine;

  /// Path of the chosen file, `null` if cancelled.
  final Future<String?> Function() pickFile;

  @override
  State<RepertoireScreen> createState() => _RepertoireScreenState();
}

Future<String?> pickScoreFile() async {
  final file = await openFile(
    acceptedTypeGroups: const [
      XTypeGroup(
        label: 'MusicXML',
        extensions: ['musicxml', 'xml', 'mxl'],
        // shortcut: iOS knows no MusicXML type, so any file; the engine rejects the rest.
        uniformTypeIdentifiers: ['public.data'],
      ),
    ],
  );
  return file?.path;
}

class _RepertoireScreenState extends State<RepertoireScreen> {
  List<Map<String, dynamic>> _pieces = [];

  @override
  void initState() {
    super.initState();
    widget.engine.repertoire().then((pieces) {
      if (mounted) setState(() => _pieces = pieces);
    });
  }

  Future<void> _loadFile() async {
    final path = await widget.pickFile();
    if (path == null || !mounted) return;
    try {
      final piece = await widget.engine.loadPiece(path);
      if (mounted) Navigator.of(context).pop(piece['id'] as String);
    } on InvalidScoreFile catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(_rejection(AppLocalizations.of(context), e))),
      );
    }
  }

  static String _rejection(AppLocalizations l10n, InvalidScoreFile e) =>
      switch (e.reason) {
        'not_partwise' => l10n.invalidNotPartwise,
        'no_pitched_notes' => l10n.invalidNoPitchedNotes,
        'unpaired_repeats' => l10n.invalidUnpairedRepeats,
        'incompatible_with_work' => l10n.invalidIncompatibleWithWork,
        _ => l10n.invalidUnreadable,
      };

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(title: Text(l10n.repertoire)),
      body: ListView(
        children: [
          for (final piece in _pieces)
            ListTile(
              title: Text(
                piece['number'] == null
                    ? piece['title'] as String
                    : l10n.pieceWithNumber(
                        piece['title'] as String,
                        piece['number'] as String,
                      ),
              ),
              subtitle: piece['has_fingering'] == true
                  ? null
                  : Text(l10n.noFingering),
              onTap: () => Navigator.of(context).pop(piece['id'] as String),
            ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: _loadFile,
        icon: const Icon(Icons.file_open),
        label: Text(l10n.loadFile),
      ),
    );
  }
}
