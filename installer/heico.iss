; Heico - Inno Setup script. Genere HeicoSetup-{version}.exe.
; Sources (heico.exe + 4 DLL) attendues dans installer\build\.

#ifndef MyAppVersion
  #define MyAppVersion "0.0.0-dev"
#endif

#define MyAppName "Heico"
#define MyAppPublisher "Bastien Brasseur"
#define MyAppURL "https://github.com/bastienbrasseur/heico"
#define MyAppExeName "heico.exe"

[Setup]
; GUID fixe pour les upgrades en place. Ne jamais changer.
AppId={{7B9E4F2A-8C1D-4B6E-9F3A-2D5C8E7B1A4F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={localappdata}\Heico
DisableDirPage=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=Output
OutputBaseFilename=HeicoSetup-{#MyAppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\heico.exe
UninstallDisplayName={#MyAppName}
SetupIconFile=..\assets\heico.ico
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Files]
Source: "build\heico.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "build\heif.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "build\libde265.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "build\libx265.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "build\dav1d.dll"; DestDir: "{app}"; Flags: ignoreversion

[Code]
const
  ExtsCount = 18;

var
  Exts: array[0..ExtsCount-1] of string;

procedure InitExts;
begin
  Exts[0]  := 'heic'; Exts[1]  := 'HEIC';
  Exts[2]  := 'heif'; Exts[3]  := 'HEIF';
  Exts[4]  := 'png';  Exts[5]  := 'PNG';
  Exts[6]  := 'webp'; Exts[7]  := 'WEBP';
  Exts[8]  := 'tif';  Exts[9]  := 'TIF';
  Exts[10] := 'tiff'; Exts[11] := 'TIFF';
  Exts[12] := 'bmp';  Exts[13] := 'BMP';
  Exts[14] := 'gif';  Exts[15] := 'GIF';
  Exts[16] := 'avif'; Exts[17] := 'AVIF';
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  i: Integer;
  KeyBase, CmdKey, ExeQ: string;
begin
  if CurStep = ssPostInstall then
  begin
    InitExts;
    ExeQ := ExpandConstant('{app}\heico.exe');
    for i := 0 to ExtsCount-1 do
    begin
      KeyBase := 'Software\Classes\SystemFileAssociations\.' + Exts[i] + '\shell\HeicoConvertToJpg';
      CmdKey  := KeyBase + '\command';
      RegWriteStringValue(HKEY_CURRENT_USER, KeyBase, '', 'Convertir en JPG');
      RegWriteStringValue(HKEY_CURRENT_USER, KeyBase, 'Icon', '"' + ExeQ + '",0');
      // Player : un seul process pour tous les fichiers selectionnes (sinon Windows masque au-dela de ~15).
      RegWriteStringValue(HKEY_CURRENT_USER, KeyBase, 'MultiSelectModel', 'Player');
      RegWriteStringValue(HKEY_CURRENT_USER, CmdKey, '', '"' + ExeQ + '" "%1"');
    end;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  i: Integer;
  KeyBase: string;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    InitExts;
    for i := 0 to ExtsCount-1 do
    begin
      KeyBase := 'Software\Classes\SystemFileAssociations\.' + Exts[i] + '\shell\HeicoConvertToJpg';
      RegDeleteKeyIncludingSubkeys(HKEY_CURRENT_USER, KeyBase);
    end;
  end;
end;
